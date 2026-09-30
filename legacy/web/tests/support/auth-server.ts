// Isolated Supabase Auth HTTP contract fixture. Never imported by application code.
import { createServer, type IncomingMessage, type ServerResponse } from "node:http";
import { createHmac, randomUUID } from "node:crypto";

const secret = "phase1-test-fixture-only-not-a-real-secret";
const user = { id: "11111111-1111-4111-8111-111111111111", email: "bookkeeper@example.test", aud: "authenticated", role: "authenticated", created_at: "2026-01-01T00:00:00Z", app_metadata: { provider: "email", providers: ["email"] }, user_metadata: {}, identities: [], is_anonymous: false };
const sessions = new Map<string, string>();
const revoked = new Set<string>();

function issueSession() {
  const now = Math.floor(Date.now() / 1000);
  const jti = randomUUID();
  const header = Buffer.from(JSON.stringify({ alg: "HS256", typ: "JWT" })).toString("base64url");
  const payload = Buffer.from(JSON.stringify({ sub: user.id, email: user.email, aud: "authenticated", role: "authenticated", iat: now, exp: now + 3600, jti })).toString("base64url");
  const signature = createHmac("sha256", secret).update(`${header}.${payload}`).digest("base64url");
  const token = `${header}.${payload}.${signature}`;
  const refresh = randomUUID();
  sessions.set(refresh, token);
  return { access_token: token, token_type: "bearer", expires_in: 3600, expires_at: now + 3600, refresh_token: refresh, user };
}

function validToken(request: IncomingMessage) {
  const token = request.headers.authorization?.replace(/^Bearer /, "");
  return token && [...sessions.values()].includes(token) && !revoked.has(token) ? token : null;
}

function send(response: ServerResponse, status: number, data: unknown) {
  response.writeHead(status, { "Content-Type": "application/json" });
  response.end(JSON.stringify(data));
}

const server = createServer(async (request, response) => {
  const url = new URL(request.url ?? "/", "http://127.0.0.1:54321");
  if (url.pathname === "/health") return send(response, 200, { ok: true });
  if (url.pathname === "/test/revoke-all") {
    sessions.forEach((token) => revoked.add(token));
    return send(response, 200, { ok: true });
  }
  let body = "";
  for await (const chunk of request) body += String(chunk);
  const input = body ? JSON.parse(body) as Record<string, string> : {};
  if (url.pathname === "/auth/v1/token" && url.searchParams.get("grant_type") === "password") {
    if (input.email === "unavailable@example.test") return send(response, 503, { message: "Test service unavailable" });
    if (input.email !== user.email || input.password !== "test-password-only") return send(response, 400, { code: "invalid_credentials", msg: "Invalid login credentials" });
    return send(response, 200, issueSession());
  }
  if (url.pathname === "/auth/v1/token" && url.searchParams.get("grant_type") === "refresh_token") {
    const previous = sessions.get(input.refresh_token);
    if (!previous || revoked.has(previous)) return send(response, 400, { code: "refresh_token_not_found", message: "Invalid refresh token" });
    revoked.add(previous);
    return send(response, 200, issueSession());
  }
  if (url.pathname === "/auth/v1/user") {
    return validToken(request) ? send(response, 200, user) : send(response, 401, { code: "bad_jwt", message: "Invalid JWT" });
  }
  if (url.pathname === "/auth/v1/logout") {
    const token = validToken(request);
    if (token) revoked.add(token);
    return send(response, 200, {});
  }
  if (url.pathname === "/auth/v1/.well-known/jwks.json") return send(response, 200, { keys: [] });
  return send(response, 404, { message: "Unknown test endpoint" });
});
server.listen(54321, "127.0.0.1", () => process.send?.("ready"));
process.on("message", (message) => {
  if (message === "stop") {
    server.closeAllConnections();
    server.close(() => process.exit(0));
  }
});
