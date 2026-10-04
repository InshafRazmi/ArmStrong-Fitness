import { Buffer } from 'node:buffer';

export class ApiError extends Error {
  status: number; code: string; details: unknown;
  constructor(status: number, code: string, details: unknown = null) {
    super(code); this.status = status; this.code = code; this.details = details;
  }
}
export function uuid(value: unknown): string {
  if (typeof value !== 'string' || !/^[0-9a-f]{8}-[0-9a-f]{4}-[1-5][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/.test(value)) throw new ApiError(400, 'invalid_uuid');
  return value;
}
function object(value: unknown, keys: string[]): Record<string, any> {
  if (!value || typeof value !== 'object' || Array.isArray(value) || Object.keys(value).some(key => !keys.includes(key)) || keys.some(key => !(key in value))) throw new ApiError(400, 'invalid_fields');
  return value as Record<string, any>;
}
function trimmed(value: unknown, max: number, empty = false): string {
  if (typeof value !== 'string' || (!empty && !value.trim()) || [...value.trim()].length > max) throw new ApiError(400, 'invalid_text');
  return value.trim();
}
export function parseEnrollment(value: unknown): { protocolVersion: 1; deviceId: string; deviceSecret: string } {
  const x = object(value, ['protocolVersion', 'deviceId', 'deviceSecret']);
  if (x.protocolVersion !== 1) throw new ApiError(400, 'invalid_protocol');
  if (typeof x.deviceSecret !== 'string' || !/^[a-f0-9]{64}$/.test(x.deviceSecret)) throw new ApiError(400, 'invalid_device_secret');
  return { protocolVersion: 1, deviceId: uuid(x.deviceId), deviceSecret: x.deviceSecret };
}
export type Member = { name: string; phone: string; email: string; nfcId: string | null; joinedOn: string };
export type Operation = { protocolVersion: 1; operationId: string; deviceId: string; memberId: string; action: 'create' | 'update' | 'archive'; expectedRevision: number; member: Member | null };
export function parseOperation(value: unknown): Operation {
  const x = object(value, ['protocolVersion', 'operationId', 'deviceId', 'memberId', 'action', 'expectedRevision', 'member']);
  if (x.protocolVersion !== 1 || !['create','update','archive'].includes(x.action) || !Number.isSafeInteger(x.expectedRevision) || x.expectedRevision < 0 || x.expectedRevision >= Number.MAX_SAFE_INTEGER || (x.action === 'create') !== (x.expectedRevision === 0)) throw new ApiError(400, 'invalid_operation');
  let member: Member | null = null;
  if (x.action === 'archive') {
    if (x.member !== null) throw new ApiError(400, 'archive_has_no_member_payload');
  } else {
    const m = object(x.member, ['name','phone','email','nfcId','joinedOn']);
    const email = trimmed(m.email, 254, true);
    if (Buffer.byteLength(email) > 254 || (email && (!email.includes('@') || /\s/.test(email)))) throw new ApiError(400, 'invalid_email');
    const card = m.nfcId === null ? '' : trimmed(m.nfcId, 128, true);
    if (card && !/^[\x21-\x7e]{1,128}$/.test(card)) throw new ApiError(400, 'invalid_card');
    if (typeof m.joinedOn !== 'string' || !/^\d{4}-\d{2}-\d{2}$/.test(m.joinedOn) || m.joinedOn < '1900-01-01' || m.joinedOn > '2200-12-31' || !Number.isFinite(Date.parse(m.joinedOn)) || new Date(m.joinedOn).toISOString().slice(0,10) !== m.joinedOn) throw new ApiError(400, 'invalid_date');
    member = { name: trimmed(m.name,120), phone: trimmed(m.phone,40), email, nfcId: card.toUpperCase() || null, joinedOn: m.joinedOn };
  }
  return { protocolVersion: 1, operationId: uuid(x.operationId), deviceId: uuid(x.deviceId), memberId: uuid(x.memberId), action: x.action, expectedRevision: x.expectedRevision, member };
}
export function cursor(value: unknown): number {
  if (typeof value !== 'string' || !/^(0|[1-9]\d*)$/.test(value) || !Number.isSafeInteger(Number(value))) throw new ApiError(400, 'invalid_cursor');
  return Number(value);
}
