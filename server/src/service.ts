import { createHash, timingSafeEqual } from 'node:crypto';
import { ApiError, parseEnrollment, parseOperation, uuid } from './protocol.ts';
import { deriveGym } from './gym.ts';
export type Scope = { gymId: string; userId: string; deviceId: string; deviceSecret: string };
// pg Pool interface kept structural so protocol/auth tests need no installed packages.
export type Pool = { connect(): Promise<any> };
const projection = `jsonb_build_object('id',m.id,'name',m.name,'phone',m.phone,'email',m.email,'nfcId',m.nfc_id,'joinedOn',to_char(m.joined_on,'YYYY-MM-DD'),'revision',m.revision,'archivedAt',CASE WHEN m.archived_at IS NULL THEN NULL ELSE to_char(m.archived_at AT TIME ZONE 'UTC','YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') END,'archivedBy',CASE WHEN m.archived_at IS NULL THEN NULL ELSE jsonb_build_object('id',s.user_id,'name',s.display_name) END)`;
export class MemberService {
  pool: Pool;
  automaticEnrollment: boolean;
  constructor(pool: Pool, automaticEnrollment = false) { this.pool = pool; this.automaticEnrollment = automaticEnrollment; }
  protected async transaction(scope: Omit<Scope, 'gymId'> & { gymId?: string }, write: boolean, run: (db: any, role: string, gymId: string) => Promise<unknown>) {
    if (scope.gymId !== undefined) uuid(scope.gymId);
    uuid(scope.deviceId); uuid(scope.userId);
    if (typeof scope.deviceSecret !== 'string' || !/^[a-f0-9]{64}$/.test(scope.deviceSecret)) throw new ApiError(403, 'device_not_authorized');
    const db = await this.pool.connect();
    try {
      await db.query('BEGIN');
      // The staff account and approved device determine access. Do not filter this
      // lookup by a client-supplied gym ID or use unverified JWT role metadata.
      const candidates = await db.query('SELECT s.gym_id,d.secret_sha256 FROM armstrong.staff s JOIN armstrong.devices d ON d.gym_id=s.gym_id WHERE s.user_id=$1 AND s.active AND d.id=$2 AND d.active', [scope.userId,scope.deviceId]);
      const gymId = deriveGym(candidates.rows, scope.deviceSecret, scope.gymId);
      // Serializes sequence allocation AND commit; BIGSERIAL alone loses late commits.
      const gym = await db.query('SELECT id FROM armstrong.gyms WHERE id=$1 FOR UPDATE', [gymId]);
      if (!gym.rowCount) throw new ApiError(403, 'gym_not_authorized');
      const staff = await db.query('SELECT role FROM armstrong.staff WHERE gym_id=$1 AND user_id=$2 AND active', [gymId, scope.userId]);
      if (!staff.rowCount) throw new ApiError(403, 'staff_not_authorized');
      const device = await db.query('SELECT secret_sha256,can_write FROM armstrong.devices WHERE gym_id=$1 AND id=$2 AND active', [gymId, scope.deviceId]);
      const digest = createHash('sha256').update(scope.deviceSecret).digest();
      if (!device.rowCount || !timingSafeEqual(digest, Buffer.from(device.rows[0].secret_sha256,'hex')) || (write && !device.rows[0].can_write)) throw new ApiError(403,'device_not_authorized');
      const result = await run(db, staff.rows[0].role, gymId);
      await db.query('COMMIT'); return result;
    } catch (error) { await db.query('ROLLBACK'); throw error; }
    finally { db.release(); }
  }
  async enroll(userId: string, body: unknown) {
    const enrollment = parseEnrollment(body);
    uuid(userId);
    if (this.automaticEnrollment) {
      const db = await this.pool.connect();
      try {
        const hash = createHash('sha256').update(enrollment.deviceSecret).digest('hex');
        const reply = await db.query('SELECT armstrong.enroll_desktop($1::uuid,$2::uuid,$3::text) AS enrollment', [userId,enrollment.deviceId,hash]);
        if (reply.rows.length !== 1 || !reply.rows[0]?.enrollment) throw new ApiError(503,'service_unavailable');
        return reply.rows[0].enrollment;
      } catch (error) {
        if ((error as {code?: string}).code === '42501') throw new ApiError(403,'registration_not_authorized');
        throw error;
      } finally { db.release(); }
    }
    // Verification of owner-approved registration only. Runtime SQL credentials
    // cannot create staff, approve devices or grant roles. No client gym selector.
    return this.transaction({ userId, deviceId: enrollment.deviceId, deviceSecret: enrollment.deviceSecret }, false, async (db, role, gymId) => {
      const gym = (await db.query('SELECT name FROM armstrong.gyms WHERE id=$1', [gymId])).rows[0];
      const staff = (await db.query('SELECT display_name FROM armstrong.staff WHERE gym_id=$1 AND user_id=$2 AND active', [gymId, userId])).rows[0];
      const device = (await db.query('SELECT can_write FROM armstrong.devices WHERE gym_id=$1 AND id=$2 AND active', [gymId, enrollment.deviceId])).rows[0];
      if (!gym || !staff || !device) throw new ApiError(403, 'registration_not_authorized');
      return { protocolVersion: 1, gym: { id: gymId, name: gym.name }, staff: { id: userId, name: staff.display_name, role }, device: { id: enrollment.deviceId, canWrite: device.can_write } };
    });
  }
  async push(scope: Scope, body: unknown) {
    const op = parseOperation(body);
    if (op.deviceId !== scope.deviceId) throw new ApiError(403, 'device_mismatch');
    try { return await this.transaction(scope, true, async (db, role, gymId) => {
      await this.beforeMemberWrite(db, gymId);
      if (op.action === 'archive' && role !== 'Administrator') throw new ApiError(403, 'administrator_required');
      const saved = await db.query('SELECT receipt,request=$3::jsonb AND device_id=$4 AND actor_user_id=$5 AS matches FROM armstrong.member_operations WHERE gym_id=$1 AND id=$2', [gymId,op.operationId,JSON.stringify(op),scope.deviceId,scope.userId]);
      if (saved.rowCount) {
        if (!saved.rows[0].matches) throw new ApiError(409, 'operation_id_reused');
        return saved.rows[0].receipt;
      }
      const select = `SELECT ${projection} AS member FROM armstrong.members m LEFT JOIN armstrong.staff s ON s.gym_id=m.gym_id AND s.user_id=m.archived_by_user_id WHERE m.gym_id=$1 AND m.id=$2`;
      const current = (await db.query(select, [gymId,op.memberId])).rows[0]?.member;
      if ((current?.revision ?? 0) !== op.expectedRevision) throw new ApiError(409,'revision_conflict',current ?? null);
      if (current?.archivedAt) throw new ApiError(409,'member_archived',current);
      if (current && op.member && current.joinedOn !== op.member.joinedOn) throw new ApiError(409,'joined_date_is_immutable',current);
      if (op.action === 'create') {
        const m = op.member!;
        await db.query('INSERT INTO armstrong.members(gym_id,id,name,phone,email,nfc_id,joined_on,revision) VALUES($1,$2,$3,$4,$5,$6,$7,1)', [gymId,op.memberId,m.name,m.phone,m.email,m.nfcId,m.joinedOn]);
      } else if (op.action === 'update') {
        const m = op.member!;
        await db.query('UPDATE armstrong.members SET name=$3,phone=$4,email=$5,nfc_id=$6,revision=revision+1 WHERE gym_id=$1 AND id=$2', [gymId,op.memberId,m.name,m.phone,m.email,m.nfcId]);
      } else {
        await db.query('UPDATE armstrong.members SET archived_at=clock_timestamp(),archived_by_user_id=$3,revision=revision+1 WHERE gym_id=$1 AND id=$2', [gymId,op.memberId,scope.userId]);
      }
      const member = (await db.query(select,[gymId,op.memberId])).rows[0].member;
      const sequence = Number((await db.query('UPDATE armstrong.gyms SET change_sequence=change_sequence+1 WHERE id=$1 RETURNING change_sequence',[gymId])).rows[0].change_sequence);
      const receipt = { protocolVersion: 1, operationId: op.operationId, memberId: op.memberId, revision: member.revision, sequence, member };
      await db.query('INSERT INTO armstrong.member_operations(gym_id,id,device_id,actor_user_id,request,receipt) VALUES($1,$2,$3,$4,$5,$6)',[gymId,op.operationId,scope.deviceId,scope.userId,JSON.stringify(op),JSON.stringify(receipt)]);
      await db.query('INSERT INTO armstrong.member_changes(gym_id,sequence,operation_id,member_id,snapshot) VALUES($1,$2,$3,$4,$5)',[gymId,sequence,op.operationId,op.memberId,JSON.stringify(member)]);
      return receipt;
    }); } catch(error) {
      if ((error as any).code === '23505') throw new ApiError(409,'card_or_member_conflict');
      throw error;
    }
  }
  protected async beforeMemberWrite(_db: any, _gymId: string): Promise<void> {}
  async pull(scope: Scope, after: number) {
    return this.transaction(scope, false, async (db, _role, gymId) => {
      const high = Number((await db.query('SELECT change_sequence FROM armstrong.gyms WHERE id=$1',[gymId])).rows[0].change_sequence);
      if (after > high) throw new ApiError(409,'cursor_ahead_of_server');
      const result = await db.query('SELECT sequence,operation_id,snapshot FROM armstrong.member_changes WHERE gym_id=$1 AND sequence>$2 ORDER BY sequence LIMIT 101',[gymId,after]);
      const changes = result.rows.slice(0,100).map((r: any) => ({ sequence: Number(r.sequence), operationId: r.operation_id, member: r.snapshot }));
      return { protocolVersion: 1, after, nextCursor: changes.at(-1)?.sequence ?? after, hasMore: result.rows.length > 100, changes };
    });
  }
}
