import { fork, type ChildProcess } from "node:child_process";
import { resolve } from "node:path";

export default async function setup() {
  const children: ChildProcess[] = [];
  async function start(file: string, env: Record<string, string> = {}) {
    const child = fork(resolve("tests/support", file), [], {
      execArgv: ["--experimental-strip-types"],
      env: { ...process.env, NEXT_TELEMETRY_DISABLED: "1", ...env },
      stdio: ["ignore", "inherit", "inherit", "ipc"],
    });
    children.push(child);
    await new Promise<void>((resolveReady, reject) => {
      const timer = setTimeout(() => reject(new Error(`Test server startup timed out: ${file}`)), 30_000);
      child.once("error", (error) => { clearTimeout(timer); reject(error); });
      child.once("exit", (code) => { clearTimeout(timer); reject(new Error(`Test server exited during startup: ${file} (${code})`)); });
      child.once("message", () => { clearTimeout(timer); resolveReady(); });
    });
  }
  async function stop() {
    await Promise.all(children.map((child) => new Promise<void>((resolveStopped, reject) => {
      if (child.exitCode !== null) return resolveStopped();
      const timer = setTimeout(() => { child.kill(); reject(new Error("Test server did not shut down cleanly")); }, 10_000);
      child.once("exit", () => { clearTimeout(timer); resolveStopped(); });
      child.send("stop");
    })));
  }
  try {
    await start("auth-server.ts");
    await start("app-server.ts", { TEST_APP_PORT: "3100", NEXT_PUBLIC_SUPABASE_URL: "http://127.0.0.1:54321", NEXT_PUBLIC_SUPABASE_PUBLISHABLE_KEY: "sb_publishable_phase1_test_only" });
    await start("app-server.ts", { TEST_APP_PORT: "3101", NEXT_PUBLIC_SUPABASE_URL: "", NEXT_PUBLIC_SUPABASE_PUBLISHABLE_KEY: "" });
  } catch (error) {
    await stop();
    throw error;
  }
  return stop;
}
