// Test-only production-server wrapper with explicit IPC shutdown on Windows.
import { createServer } from "node:http";
import next from "next";

const port = Number(process.env.TEST_APP_PORT);
const app = next({ dev: false, hostname: "127.0.0.1", port });
await app.prepare();
const handler = app.getRequestHandler();
const server = createServer((request, response) => { void handler(request, response); });
server.listen(port, "127.0.0.1", () => process.send?.("ready"));
process.on("message", (message) => {
  if (message === "stop") {
    server.closeAllConnections();
    server.close(() => { void app.close().then(() => process.exit(0)); });
  }
});
