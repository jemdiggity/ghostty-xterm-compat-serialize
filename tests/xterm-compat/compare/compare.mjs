import { execFile } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { promisify } from "node:util";

const execFileAsync = promisify(execFile);
const __dirname = path.dirname(fileURLToPath(import.meta.url));
const rootDir = path.resolve(__dirname, "../../..");
const referenceRunnerPath = path.resolve(
  rootDir,
  "tests/xterm-compat/node-runner/reference-runner.mjs",
);
const ghosttyRunnerDir = path.resolve(rootDir, "tests/xterm-compat/ghostty-runner");
const CONTEXT_RADIUS = 8;

function sliceContext(text, index) {
  const start = Math.max(0, index - CONTEXT_RADIUS);
  const end = Math.min(text.length, index + CONTEXT_RADIUS + 1);
  return text.slice(start, end);
}

function describeChar(text, index) {
  if (index >= text.length) {
    return null;
  }
  return text[index];
}

export function findFirstMismatch(referenceSerialized, candidateSerialized) {
  const maxLength = Math.max(referenceSerialized.length, candidateSerialized.length);

  for (let index = 0; index < maxLength; index += 1) {
    if (referenceSerialized[index] === candidateSerialized[index]) {
      continue;
    }

    return {
      index,
      referenceChar: describeChar(referenceSerialized, index),
      candidateChar: describeChar(candidateSerialized, index),
      referenceContext: sliceContext(referenceSerialized, index),
      candidateContext: sliceContext(candidateSerialized, index),
    };
  }

  return null;
}

export function compareResults(reference, candidate) {
  const exactMatch = reference.serialized === candidate.serializedCandidate;
  const semanticMatch =
    reference.cursorX === candidate.cursorX &&
    reference.cursorY === candidate.cursorY;
  const firstMismatch = exactMatch
    ? null
    : findFirstMismatch(reference.serialized, candidate.serializedCandidate);

  return {
    exactMatch,
    semanticMatch,
    serializedDiffers: !exactMatch,
    firstMismatch,
  };
}

async function runJsonCommand(command, args, options = {}) {
  const { stdout } = await execFileAsync(command, args, {
    cwd: rootDir,
    maxBuffer: 10 * 1024 * 1024,
    ...options,
  });
  return JSON.parse(stdout);
}

export async function compareFixture(name) {
  const reference = await runJsonCommand("node", [referenceRunnerPath, name]);
  const candidate = await runJsonCommand("cargo", ["run", "--quiet", "--", name], {
    cwd: ghosttyRunnerDir,
  });
  return {
    fixture: name,
    reference,
    candidate,
    ...compareResults(reference, candidate),
  };
}
