import fs from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { SerializeAddon } from "@xterm/addon-serialize";
import headless from "@xterm/headless";

const { Terminal } = headless;

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const fixturesDir = path.resolve(__dirname, "../fixtures");

async function loadFixture(name) {
  const filePath = path.join(fixturesDir, `${name}.json`);
  const raw = await fs.readFile(filePath, "utf8");
  return JSON.parse(raw);
}

async function writeChunk(terminal, data) {
  await new Promise((resolve) => {
    terminal.write(data, resolve);
  });
}

export async function runReferenceFixture(name) {
  const fixture = await loadFixture(name);
  const terminal = new Terminal({
    cols: 80,
    rows: 24,
    scrollback: 1000,
  });
  const serializeAddon = new SerializeAddon();

  terminal.loadAddon(serializeAddon);

  for (const chunk of fixture.chunks) {
    if (chunk.delayMs > 0) {
      await new Promise((resolve) => setTimeout(resolve, chunk.delayMs));
    }

    await writeChunk(terminal, chunk.data);
  }

  return {
    fixture: fixture.name,
    serialized: serializeAddon.serialize(),
    cursorX: terminal.buffer.active.cursorX,
    cursorY: terminal.buffer.active.cursorY,
    baseY: terminal.buffer.active.baseY,
    viewportY: terminal.buffer.active.viewportY,
  };
}

if (process.argv[1] === fileURLToPath(import.meta.url) && process.argv[2]) {
  const result = await runReferenceFixture(process.argv[2]);
  process.stdout.write(`${JSON.stringify(result, null, 2)}\n`);
}
