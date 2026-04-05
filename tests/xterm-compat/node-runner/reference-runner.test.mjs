import assert from "node:assert/strict";
import test from "node:test";

import { runReferenceFixture } from "./reference-runner.mjs";

test("startup_prompt fixture produces serialized output and cursor metadata", async () => {
  const result = await runReferenceFixture("startup_prompt");

  assert.equal(typeof result.serialized, "string");
  assert.ok(result.serialized.includes("OpenAI Codex"));
  assert.equal(typeof result.cursorX, "number");
  assert.equal(typeof result.cursorY, "number");
});
