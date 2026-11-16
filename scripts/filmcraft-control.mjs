// Minimal FilmCraft control-channel client (JSON lines over loopback TCP).
// Usage:
//   node scripts/filmcraft-control.mjs smoke
//   node scripts/filmcraft-control.mjs call engine.commands
//
// Requires FILMCRAFT_CONTROL_TOKEN in the environment (same value the app was started with).

import net from "node:net";
import path from "node:path";
import fs from "node:fs";

const HOST = process.env.FILMCRAFT_CONTROL_HOST ?? "127.0.0.1";
const PORT = Number(process.env.FILMCRAFT_CONTROL_PORT ?? 9876);
const TOKEN = process.env.FILMCRAFT_CONTROL_TOKEN ?? "";

if (!TOKEN) {
  console.error("FILMCRAFT_CONTROL_TOKEN is not set");
  process.exit(2);
}

function connect() {
  return new Promise((resolve, reject) => {
    const socket = net.createConnection({ host: HOST, port: PORT }, () => resolve(socket));
    socket.once("error", reject);
  });
}

class Client {
  constructor(socket) {
    this.socket = socket;
    this.buffer = "";
    this.pending = new Map();
    this.nextId = 1;
    socket.setEncoding("utf8");
    socket.on("data", (chunk) => this.onData(chunk));
  }

  onData(chunk) {
    this.buffer += chunk;
    let index;
    while ((index = this.buffer.indexOf("\n")) >= 0) {
      const line = this.buffer.slice(0, index).trim();
      this.buffer = this.buffer.slice(index + 1);
      if (!line) continue;
      let message;
      try {
        message = JSON.parse(line);
      } catch {
        console.error("non-JSON reply:", line);
        continue;
      }
      const waiter = this.pending.get(message.id);
      if (waiter) {
        this.pending.delete(message.id);
        waiter(message);
      }
    }
  }

  call(method, params = {}) {
    const id = this.nextId++;
    const payload = JSON.stringify({ id, token: TOKEN, method, params });
    return new Promise((resolve) => {
      this.pending.set(id, resolve);
      this.socket.write(payload + "\n");
    });
  }
}

const outDir = path.resolve("vendor/out");
fs.mkdirSync(outDir, { recursive: true });

const socket = await connect();
const client = new Client(socket);

const [mode, method, ...rest] = process.argv.slice(2);

if (mode === "call") {
  // PowerShell mangles inline JSON quotes; prefer FILMCRAFT_PARAMS for anything non-trivial.
  const raw = process.env.FILMCRAFT_PARAMS ?? rest.join(" ");
  const params = raw.trim() ? JSON.parse(raw) : {};
  console.log(JSON.stringify(await client.call(method, params), null, 2));
} else if (mode === "smoke") {
  const log = (label, value) => console.log(`${label}: ${value}`);

  const t0 = Date.now();
  const commands = await client.call("engine.commands");
  log("engine.commands", `${commands.ok ? commands.result.length : commands.error} (${Date.now() - t0} ms)`);

  const open = await client.call("engine.execute", { command: "file.openDemoProject" });
  log("openDemoProject", open.ok ? "ok" : open.error);

  const seq = await client.call("engine.execute", { command: "sequence.inspect" });
  if (!seq.ok) {
    log("sequence.inspect", seq.error);
  } else {
    const s = seq.result.sequence ?? seq.result;
    const tracks = s.tracks ?? [...(s.video ?? []), ...(s.audio ?? [])];
    log("sequence", `${s.name ?? "?"} · ${s.frameRate ?? "?"} fps · ${s.durationFrames ?? "?"} frames`);
    log("tracks", `${tracks.length} (${(s.video ?? []).length} video, ${(s.audio ?? []).length} audio)`);
    const clips = tracks.flatMap((t) => t.clips ?? []);
    log("clips", `${clips.length}`);
    if (clips[0]) log("first clip", JSON.stringify(clips[0]).slice(0, 300));
  }

  const ui = await client.call("ui.inspect");
  log("ui.inspect", ui.ok ? JSON.stringify(ui.result).slice(0, 400) : ui.error);

  const shot = path.join(outDir, "app.png");
  const snap = await client.call("ui.screenshot", { path: shot });
  log("ui.screenshot", snap.ok ? shot : snap.error);

  const panel = path.join(outDir, "timeline.png");
  const snap2 = await client.call("ui.screenshot", { path: panel, panel: "Timeline" });
  log("timeline screenshot", snap2.ok ? panel : snap2.error);

  // A real edit: move the playhead, then report where the app thinks it is.
  const seek = await client.call("engine.execute", { command: "playhead.set", params: { seconds: 3 } });
  log("playhead.set(3s)", seek.ok ? JSON.stringify(seek.result) : seek.error);
} else {
  console.error("usage: node scripts/filmcraft-control.mjs smoke | call <method> [json params]");
  process.exitCode = 2;
}

socket.end();