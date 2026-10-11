import Fastify from 'fastify';
import { ApiError, cursor, uuid } from './protocol.ts';
import type { MemberService, Scope } from './service.ts';
import { apiHealth } from './api-check.ts';
import type { GymService } from './business-service.ts';
import { REQUEST_LIMIT } from './business-protocol.ts';
type Service = Pick<MemberService, 'enroll' | 'push' | 'pull'> & Partial<Pick<GymService, 'pushBusiness' | 'pullBusiness'>>;
export function createApp(service: Service, verify: (header: unknown) => Promise<string>) {
  const app = Fastify({ bodyLimit: 32768, logger: false, requestTimeout: 15000 });
  async function scope(request: any): Promise<Scope> {
    const userId = await verify(request.headers.authorization);
    return { userId, gymId: uuid(request.headers['x-gym-id']), deviceId: uuid(request.headers['x-device-id']), deviceSecret: request.headers['x-device-secret'] };
  }
  app.addHook('onSend', async (_request, reply, payload) => {
    reply.header('cache-control', 'no-store');
    reply.header('x-content-type-options', 'nosniff');
    return payload;
  });
  app.get('/health', async () => apiHealth);
  app.post('/v1/enrollment', async request => service.enroll(await verify(request.headers.authorization), request.body));
  app.post('/v1/members/push', async request => service.push(await scope(request), request.body));
  app.get('/v1/members/changes', async request => service.pull(await scope(request), cursor((request.query as any).after)));
  if (service.pushBusiness && service.pullBusiness) {
    app.get('/v2/health', async () => ({ status: 'ok', service: 'armstrong-gym-api', protocolVersion: 2, businessSchemaVersion: 12 }));
    app.post('/v2/business/push', { bodyLimit: REQUEST_LIMIT }, async request => service.pushBusiness!(await scope(request), request.body));
    app.get('/v2/business/changes', async request => service.pullBusiness!(await scope(request), cursor((request.query as any).after)));
  }
  app.setErrorHandler((error, _request, reply) => {
    if (error instanceof ApiError) return reply.code(error.status).send({ error: error.code, details: error.details });
    const status = error && typeof error === 'object' && 'statusCode' in error ? error.statusCode : undefined;
    if (typeof status === 'number' && status >= 400 && status < 500) return reply.code(status).send({ error: 'invalid_request' });
    // Do not expose PostgreSQL details, bearer tokens, device secrets or member PII.
    return reply.code(503).send({ error: 'service_unavailable' });
  });
  return app;
}
