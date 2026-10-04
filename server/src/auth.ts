import { ApiError, uuid } from "./protocol.ts";
// Never decode an unverified JWT and trust its sub/role. Supabase verifies it online.
export function createVerifier(
  url: string,
  key: string,
  request: typeof fetch = fetch,
) {
  const endpoint = new URL("/auth/v1/user", url);
  if (endpoint.protocol !== "https:" || endpoint.username || endpoint.password)
    throw new Error("SUPABASE_URL must be HTTPS");
  return async (authorization: unknown): Promise<string> => {
    if (
      typeof authorization !== "string" ||
      !/^Bearer [A-Za-z0-9._~-]+$/.test(authorization) ||
      authorization.length > 8192
    )
      throw new ApiError(401, "authentication_required");
    let response: Response;
    try {
      response = await request(endpoint, {
        headers: { authorization, apikey: key },
        signal: AbortSignal.timeout(10000),
        redirect: "error",
      });
    } catch {
      throw new ApiError(503, "identity_unavailable");
    }
    if (response.status === 401 || response.status === 403)
      throw new ApiError(401, "invalid_session");
    if (!response.ok) throw new ApiError(503, "identity_unavailable");
    try {
      const identity: unknown = await response.json();
      return uuid(
        identity && typeof identity === "object"
          ? (identity as Record<string, unknown>).id
          : undefined,
      );
    } catch {
      throw new ApiError(503, "invalid_identity_response");
    }
  };
}
