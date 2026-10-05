import { ApiError } from './protocol.ts';
import { MemberService } from './service.ts';
import type { Scope } from './service.ts';
import { applyChanges, byTable, digest, parseBatch, stateFrom, REQUEST_LIMIT } from './business-protocol.ts';

export class GymService extends MemberService {
  protected override async beforeMemberWrite(db: any, gymId: string) {
    const gym = (await db.query('SELECT business_sequence FROM armstrong.gyms WHERE id=$1', [gymId])).rows[0];
    if (Number(gym.business_sequence) > 0) throw new ApiError(409, 'protocol_upgrade_required');
  }
  async pushBusiness(scope: Scope, body: unknown) {
    const batch = parseBatch(body);
    if (batch.deviceId !== scope.deviceId || (batch.actorSubject !== null && batch.actorSubject !== scope.userId)) throw new ApiError(403, 'business_actor_mismatch');
    try {
      return await this.transaction(scope, true, async (db, role, gymId) => {
        if (role !== 'Administrator') throw new ApiError(403, 'administrator_required');
        const saved = await db.query('SELECT receipt,request=$3::jsonb AND device_id=$4 AND actor_user_id=$5 AS matches FROM armstrong.business_operations WHERE gym_id=$1 AND id=$2', [gymId, batch.operationId, JSON.stringify(batch), scope.deviceId, scope.userId]);
        if (saved.rowCount) {
          if (!saved.rows[0].matches) throw new ApiError(409, 'operation_id_reused');
          return saved.rows[0].receipt;
        }
        const legacy = await db.query("SELECT EXISTS(SELECT 1 FROM armstrong.members WHERE gym_id=$1) AND NOT EXISTS(SELECT 1 FROM armstrong.business_records WHERE gym_id=$1 AND table_name='members') AS unresolved", [gymId]);
        if (legacy.rows[0]?.unresolved) throw new ApiError(409, 'legacy_member_reconciliation_required');
        const existing = (await db.query('SELECT table_name,record_id,data FROM armstrong.business_records WHERE gym_id=$1', [gymId])).rows;
        const proposed = stateFrom(existing);
        // Complete state validation precedes writes: a failed invoice/payment,
        // sale/stock or foreign reference never leaves a partially accepted group.
        applyChanges(proposed, batch.changes);
        for (const c of batch.changes) {
          await db.query('INSERT INTO armstrong.business_records(gym_id,table_name,record_id,data) VALUES($1,$2,$3,$4) ON CONFLICT(gym_id,table_name,record_id) DO UPDATE SET data=EXCLUDED.data WHERE armstrong.business_records.data<>EXCLUDED.data', [gymId, c.table, c.id, JSON.stringify(c.after)]);
        }
        for (const c of batch.changes) for (const ref of byTable.get(c.table)!.references) {
          if (c.after[ref.column] !== null) await db.query('INSERT INTO armstrong.business_references(gym_id,table_name,record_id,column_name,target_table,target_id) VALUES($1,$2,$3,$4,$5,$6) ON CONFLICT(gym_id,table_name,record_id,column_name) DO UPDATE SET target_id=EXCLUDED.target_id', [gymId, c.table, c.id, ref.column, ref.table, String(c.after[ref.column])]);
        }
        const sequence = Number((await db.query('UPDATE armstrong.gyms SET business_sequence=business_sequence+1 WHERE id=$1 RETURNING business_sequence', [gymId])).rows[0].business_sequence);
        const receipt = { protocolVersion: 2, operationId: batch.operationId, deviceId: batch.deviceId, gymId, actorSubject: scope.userId, sequence, requestSha256: digest(batch) };
        await db.query('INSERT INTO armstrong.business_operations(gym_id,id,device_id,actor_user_id,request,receipt) VALUES($1,$2,$3,$4,$5,$6)', [gymId, batch.operationId, batch.deviceId, scope.userId, JSON.stringify(batch), JSON.stringify(receipt)]);
        await db.query('INSERT INTO armstrong.business_changes(gym_id,sequence,operation_id) VALUES($1,$2,$3)', [gymId, sequence, batch.operationId]);
        return receipt;
      });
    } catch (error) {
      if (['23505', '23503', '23514'].includes((error as { code?: string }).code ?? '')) throw new ApiError(409, 'business_constraint_conflict');
      throw error;
    }
  }
  async pullBusiness(scope: Scope, after: number) {
    return this.transaction(scope, false, async (db, _role, gymId) => {
      const legacy = await db.query("SELECT EXISTS(SELECT 1 FROM armstrong.members WHERE gym_id=$1) AND NOT EXISTS(SELECT 1 FROM armstrong.business_records WHERE gym_id=$1 AND table_name='members') AS unresolved", [gymId]);
      if (legacy.rows[0]?.unresolved) throw new ApiError(409, 'legacy_member_reconciliation_required');
      const high = Number((await db.query('SELECT business_sequence FROM armstrong.gyms WHERE id=$1', [gymId])).rows[0].business_sequence);
      if (after > high) throw new ApiError(409, 'cursor_ahead_of_server');
      // One atomic group per bounded page; never split sale/payment effects.
      const result = await db.query('SELECT c.sequence,o.request,o.receipt FROM armstrong.business_changes c JOIN armstrong.business_operations o ON o.gym_id=c.gym_id AND o.id=c.operation_id WHERE c.gym_id=$1 AND c.sequence>$2 ORDER BY c.sequence LIMIT 2', [gymId, after]);
      const first = result.rows[0];
      const changes = first ? [{ sequence: Number(first.sequence), request: first.request, receipt: first.receipt }] : [];
      const reply = { protocolVersion: 2, gymId, after, nextCursor: first ? Number(first.sequence) : after, hasMore: result.rows.length > 1, changes };
      if (Buffer.byteLength(JSON.stringify(reply)) > REQUEST_LIMIT + 8192) throw new ApiError(503, 'business_page_too_large');
      return reply;
    });
  }
}
