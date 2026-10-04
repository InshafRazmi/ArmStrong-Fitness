import { createHash, timingSafeEqual } from 'node:crypto';
import { ApiError } from './protocol.ts';
// Rows must come from the server-side join of VERIFIED staff and approved devices.
// The requested gym is an assertion checked AFTER deriving account/device access.
export function deriveGym(rows: {gym_id: string; secret_sha256: string}[], deviceSecret: string, assertedGym?: string): string {
  const digest = createHash('sha256').update(deviceSecret).digest();
  const matches = rows.filter(row => /^[a-f0-9]{64}$/.test(row.secret_sha256) && timingSafeEqual(digest, Buffer.from(row.secret_sha256,'hex')));
  if (matches.length !== 1) throw new ApiError(403, 'device_not_authorized');
  const gymId = matches[0].gym_id;
  if (assertedGym !== undefined && gymId !== assertedGym) throw new ApiError(403, 'gym_not_authorized');
  return gymId;
}
