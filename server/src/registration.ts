import { uuid } from './protocol.ts';

export class RegistrationError extends Error {}
export type Registration = {
  gymId: string; gymName: string; adminName: string;
  device?: { id: string; secretSha256: string };
};
type Client = { query(sql: string, values?: unknown[]): Promise<{ rows: Record<string, unknown>[] }> };
type State = 'create' | 'existing';
export type RegistrationPlan = { gym: State; administrator: State; device: State | 'not included' };

function fail(message: string): never { throw new RegistrationError(`${message}; values withheld; existing records retained`); }
function text(value: unknown): string {
  if (typeof value !== 'string' || !value.trim() || [...value.trim()].length > 120) fail('Registration names must contain 1–120 characters');
  return value.trim();
}

// Only an administrative CLI calls this. The user ID comes from online Auth
// verification, never from a webview, request, env role claim or supplied UUID.
// No UPDATE, DELETE, role reassignment, secret rotation or migration occurs.
export async function registerAdministrator(client: Client, verifiedUserId: string, requested: Registration, apply = false): Promise<RegistrationPlan> {
  let userId: string, gymId: string, deviceId: string | undefined;
  try { userId = uuid(verifiedUserId); gymId = uuid(requested.gymId); if (requested.device) deviceId = uuid(requested.device.id); }
  catch { return fail('Registration requires canonical verified identity/gym/device UUIDs'); }
  const gymName = text(requested.gymName), adminName = text(requested.adminName);
  if (requested.device && (typeof requested.device.secretSha256 !== 'string' || !/^[a-f0-9]{64}$/.test(requested.device.secretSha256))) fail('Device approval requires a SHA-256 hash of its native credential');

  await client.query(apply ? 'BEGIN' : 'BEGIN ISOLATION LEVEL REPEATABLE READ READ ONLY');
  try {
    if (apply) await client.query('SELECT pg_advisory_xact_lock(714339806)');
    // Confirm the online-verified subject exists in this SQL project's Auth
    // records. Read only the boolean, never emails, passwords or Auth payloads.
    const identity = (await client.query('SELECT EXISTS(SELECT 1 FROM auth.users WHERE id=$1 AND email_confirmed_at IS NOT NULL) AS confirmed', [userId])).rows[0];
    if (identity?.confirmed !== true) fail('Verified account must be confirmed in the runtime database project');
    const gyms = (await client.query(`SELECT id,name FROM armstrong.gyms WHERE id=$1${apply ? ' FOR UPDATE' : ''}`, [gymId])).rows;
    if (gyms.length > 1 || (gyms.length === 1 && gyms[0].name !== gymName)) fail('Existing gym differs from the approved registration');
    const staff = (await client.query("SELECT gym_id,user_id,display_name,role,active FROM armstrong.staff WHERE user_id=$1 OR (gym_id=$2 AND role='Administrator')", [userId, gymId])).rows;
    if (staff.length > 1 || (staff.length === 1 && (staff[0].gym_id !== gymId || staff[0].user_id !== userId || staff[0].display_name !== adminName || staff[0].role !== 'Administrator' || staff[0].active !== true))) fail('Existing identity or Administrator permission differs; review registration explicitly');
    let devices: Record<string, unknown>[] = [];
    if (requested.device) {
      devices = (await client.query('SELECT gym_id,id,secret_sha256,active,can_write FROM armstrong.devices WHERE id=$1 OR (gym_id=$2 AND active AND can_write)', [deviceId, gymId])).rows;
      if (devices.length > 1 || (devices.length === 1 && (devices[0].gym_id !== gymId || devices[0].id !== deviceId || devices[0].secret_sha256 !== requested.device.secretSha256 || devices[0].active !== true || devices[0].can_write !== true))) fail('Existing device or writer approval differs; review registration explicitly');
    }
    const plan: RegistrationPlan = { gym: gyms.length ? 'existing' : 'create', administrator: staff.length ? 'existing' : 'create', device: requested.device ? (devices.length ? 'existing' : 'create') : 'not included' };
    if (apply) {
      if (plan.gym === 'create') await client.query('INSERT INTO armstrong.gyms(id,name) VALUES($1,$2)', [gymId, gymName]);
      if (plan.administrator === 'create') await client.query("INSERT INTO armstrong.staff(gym_id,user_id,display_name,role,active) VALUES($1,$2,$3,'Administrator',true)", [gymId, userId, adminName]);
      if (requested.device && plan.device === 'create') await client.query('INSERT INTO armstrong.devices(gym_id,id,secret_sha256,active,can_write) VALUES($1,$2,$3,true,true)', [gymId, deviceId, requested.device.secretSha256]);
    }
    await client.query('COMMIT');
    return plan;
  } catch (error) {
    try { await client.query('ROLLBACK'); }
    catch { throw new RegistrationError('Registration rollback failed; details withheld; inspect database before retrying'); }
    throw error;
  }
}
