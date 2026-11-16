// MCP stdio smoke test against the running FilmCraft app (bridge mode).
// Spawns `filmcraft-cli mcp --bridge 127.0.0.1:9876`, then: initialize → tools/list → tools/call.
// Requires FILMCRAFT_CONTROL_TOKEN (same value the app was started with).

import { spawn } from "node:child_process";

const CLI = process.env.FILMCRAFT_CLI ?? "vendor/filmcraft/filmcraft-0.20.1-windows-x64-portable/filmcraft-cli.exe";
const BRIDGE = process.env.FILMCRAFT_BRIDGE ?? "127.0.0.1:9876";
const TOKEN = process.env.FILMCRAFT_CONTROL_TOKEN ?? "";

const child = spawn(CLI, ["mcp", "--bridge", BRIDGE], {
  env: { ...process.env, FILMCRAFT_CONTROL_TOKEN: TOKEN },
  stdio: ["pipe", "pipe", "inherit"],
});

let buffer = "";
const pending = new Map();

child.stdout.setEncoding("utf8");
child.stdout.on("data", (chunk) => {
  buffer += chunk;
  let i;
  while ((i = buffer.indexOf("\n")) >= 0) {
    const line = buffer.slice(0, i).trim();
    buffer = buffer.slice(i + 1);
    if (!line) continue;
    let msg;
    try {
      msg = JSON.parse(line);
    } catch {
      console.log("[non-json]", line.slice(0, 200));
      continue;
    }
    if (msg.id && pending.has(msg.id)) {
      pending.get(msg.id)(msg);
      pending.delete(msg.id);
    }
  }
});

function send(method, params, id) {
  const payload = { jsonrpc: "2.0", method, params };
  if (id !== undefined) payload.id = id;
  child.stdin.write(JSON.stringify(payload) + "\n");
  if (id === undefined) return Promise.resolve();
  return new Promise((resolve) => pending.set(id, resolve));
}

const init = await send(
  "initialize",
  {
    protocolVersion: "2025-06-18",
    capabilities: {},
    clientInfo: { name: "agentic-editor-smoke", version: "0.0.1" },
  },
  1,
);
console.log("server:", JSON.stringify(init.result?.serverInfo ?? init.error));
console.log("protocol:", init.result?.protocolVersion);

send("notifications/initialized", {});

const tools = await send("tools/list", {}, 2);
const names = (tools.result?.tools ?? []).map((t) => t.name);
console.log(`tools (${names.length}):`, names.join(", "));

const call = await send(
  "tools/call",
  { name: "sequence_inspect", arguments: {} },
  3,
);
const text = call.result?.content?.[0]?.text ?? "";
console.log("sequence_inspect isError:", call.result?.isError);
console.log("sequence_inspect payload:", text.slice(0, 240).replace(/\s+/g, " "));

const run = await send(
  "tools/call",
  {
    name: "command_run",
    arguments: { id: "playhead.set", params: { seconds: 1.5 } },
  },
  4,
);
console.log("command_run:", JSON.stringify(run.result?.content?.[0]?.text ?? run.error).slice(0, 200));

// Vision feedback loop: the agent must be able to *see* the frame it just edited.
const frame = await send("tools/call", { name: "render_frame", arguments: { seconds: 2 } }, 5);
const blocks = frame.result?.content ?? [];
console.log(
  "render_frame blocks:",
  blocks.map((b) => `${b.type}${b.mimeType ? `:${b.mimeType}` : ""}(${b.data?.length ?? b.text?.length ?? 0})`).join(", "),
);
const image = blocks.find((b) => b.type === "image" && b.data);
if (image) {
  const { writeFileSync, mkdirSync } = await import("node:fs");
  mkdirSync("vendor/out", { recursive: true });
  writeFileSync("vendor/out/mcp-frame.png", Buffer.from(image.data, "base64"));
  console.log("wrote vendor/out/mcp-frame.png");
}

child.stdin.end();
child.kill();