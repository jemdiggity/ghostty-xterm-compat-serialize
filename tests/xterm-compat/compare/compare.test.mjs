import assert from "node:assert/strict";
import test from "node:test";

import { compareFixture, compareResults } from "./compare.mjs";

test("compareResults distinguishes exact and semantic matches", () => {
  const result = compareResults(
    { serialized: "abc", cursorX: 1, cursorY: 2 },
    { serializedCandidate: "abd", cursorX: 1, cursorY: 2 },
  );

  assert.equal(result.exactMatch, false);
  assert.equal(result.semanticMatch, true);
  assert.deepEqual(result.firstMismatch, {
    index: 2,
    referenceChar: "c",
    candidateChar: "d",
    referenceContext: "abc",
    candidateContext: "abd",
  });
});

test("compareFixture runs the reference and ghostty runners for startup_prompt", async () => {
  const result = await compareFixture("startup_prompt");

  assert.equal(result.fixture, "startup_prompt");
  assert.equal(typeof result.reference.serialized, "string");
  assert.equal(typeof result.candidate.serializedCandidate, "string");
  assert.equal(result.semanticMatch, true);
});

test("startup_prompt is an exact serialize match after xterm-compat normalization", async () => {
  const result = await compareFixture("startup_prompt");

  assert.equal(result.exactMatch, true);
  assert.equal(result.serializedDiffers, false);
});

test("prompt_redraw is an exact serialize match after row normalization", async () => {
  const result = await compareFixture("prompt_redraw");

  assert.equal(result.fixture, "prompt_redraw");
  assert.equal(result.semanticMatch, true);
  assert.equal(result.exactMatch, true);
  assert.equal(result.serializedDiffers, false);
});

test("prompt_redraw_offset is an exact serialize match with a shifted redraw column", async () => {
  const result = await compareFixture("prompt_redraw_offset");

  assert.equal(result.fixture, "prompt_redraw_offset");
  assert.equal(result.semanticMatch, true);
  assert.equal(result.exactMatch, true);
  assert.equal(result.serializedDiffers, false);
});

test("multiline_output is an exact serialize match after soft-wrap normalization", async () => {
  const result = await compareFixture("multiline_output");

  assert.equal(result.fixture, "multiline_output");
  assert.equal(result.semanticMatch, true);
  assert.equal(result.exactMatch, true);
  assert.equal(result.serializedDiffers, false);
});

test("scrollback_output is an exact serialize match when content exceeds the viewport", async () => {
  const result = await compareFixture("scrollback_output");

  assert.equal(result.fixture, "scrollback_output");
  assert.equal(result.semanticMatch, true);
  assert.equal(result.exactMatch, true);
  assert.equal(result.serializedDiffers, false);
});

test("scrollback_styled_output is an exact serialize match when styled content exceeds the viewport", async () => {
  const result = await compareFixture("scrollback_styled_output");

  assert.equal(result.fixture, "scrollback_styled_output");
  assert.equal(result.semanticMatch, true);
  assert.equal(result.exactMatch, true);
  assert.equal(result.serializedDiffers, false);
});

test("alternate_screen is an exact serialize match while preserving inactive primary content", async () => {
  const result = await compareFixture("alternate_screen");

  assert.equal(result.fixture, "alternate_screen");
  assert.equal(result.semanticMatch, true);
  assert.equal(result.exactMatch, true);
  assert.equal(result.serializedDiffers, false);
});

test("cursor_hidden_bar is an exact serialize match for cursor visibility and style state", async () => {
  const result = await compareFixture("cursor_hidden_bar");

  assert.equal(result.fixture, "cursor_hidden_bar");
  assert.equal(result.semanticMatch, true);
  assert.equal(result.exactMatch, true);
  assert.equal(result.serializedDiffers, false);
});

test("alternate_screen_scrollback is an exact serialize match with preserved primary history", async () => {
  const result = await compareFixture("alternate_screen_scrollback");

  assert.equal(result.fixture, "alternate_screen_scrollback");
  assert.equal(result.semanticMatch, true);
  assert.equal(result.exactMatch, true);
  assert.equal(result.serializedDiffers, false);
});

test("cursor_shape_only is an exact serialize match for visible non-default cursor shape state", async () => {
  const result = await compareFixture("cursor_shape_only");

  assert.equal(result.fixture, "cursor_shape_only");
  assert.equal(result.semanticMatch, true);
  assert.equal(result.exactMatch, true);
  assert.equal(result.serializedDiffers, false);
});

test("scroll_region_active is an exact serialize match with a non-default scrolling region", async () => {
  const result = await compareFixture("scroll_region_active");

  assert.equal(result.fixture, "scroll_region_active");
  assert.equal(result.semanticMatch, true);
  assert.equal(result.exactMatch, true);
  assert.equal(result.serializedDiffers, false);
});

test("origin_mode_active is an exact serialize match with origin mode enabled", async () => {
  const result = await compareFixture("origin_mode_active");

  assert.equal(result.fixture, "origin_mode_active");
  assert.equal(result.semanticMatch, true);
  assert.equal(result.exactMatch, true);
  assert.equal(result.serializedDiffers, false);
});

test("left_right_margin_active exposes an expected xterm incompatibility", async () => {
  const result = await compareFixture("left_right_margin_active");

  assert.equal(result.fixture, "left_right_margin_active");
  assert.equal(result.semanticMatch, false);
  assert.equal(result.exactMatch, false);
  assert.equal(result.serializedDiffers, true);
});

test("wraparound_disabled is an exact serialize match with DECAWM disabled", async () => {
  const result = await compareFixture("wraparound_disabled");

  assert.equal(result.fixture, "wraparound_disabled");
  assert.equal(result.semanticMatch, true);
  assert.equal(result.exactMatch, true);
  assert.equal(result.serializedDiffers, false);
});

test("reverse_wrap_enabled is an exact serialize match with reverse wrap enabled", async () => {
  const result = await compareFixture("reverse_wrap_enabled");

  assert.equal(result.fixture, "reverse_wrap_enabled");
  assert.equal(result.semanticMatch, true);
  assert.equal(result.exactMatch, true);
  assert.equal(result.serializedDiffers, false);
});

test("synchronized_output_enabled matches xterm's ignored synchronized-output state", async () => {
  const result = await compareFixture("synchronized_output_enabled");

  assert.equal(result.fixture, "synchronized_output_enabled");
  assert.equal(result.semanticMatch, true);
  assert.equal(result.exactMatch, true);
  assert.equal(result.serializedDiffers, false);
});

test("save_restore_with_style is an exact serialize match for saved cursor style state", async () => {
  const result = await compareFixture("save_restore_with_style");

  assert.equal(result.fixture, "save_restore_with_style");
  assert.equal(result.semanticMatch, true);
  assert.equal(result.exactMatch, true);
  assert.equal(result.serializedDiffers, false);
});

test("save_restore_cursor_cross_line is an exact serialize match for saved cursor position", async () => {
  const result = await compareFixture("save_restore_cursor_cross_line");

  assert.equal(result.fixture, "save_restore_cursor_cross_line");
  assert.equal(result.semanticMatch, true);
  assert.equal(result.exactMatch, true);
  assert.equal(result.serializedDiffers, false);
});

test("palette_override_ansi_color is an exact serialize match for OSC 4 palette overrides", async () => {
  const result = await compareFixture("palette_override_ansi_color");

  assert.equal(result.fixture, "palette_override_ansi_color");
  assert.equal(result.semanticMatch, true);
  assert.equal(result.exactMatch, true);
  assert.equal(result.serializedDiffers, false);
});

test("alt_save_restore_cursor_in_alt is an exact serialize match in the alternate screen", async () => {
  const result = await compareFixture("alt_save_restore_cursor_in_alt");

  assert.equal(result.fixture, "alt_save_restore_cursor_in_alt");
  assert.equal(result.semanticMatch, true);
  assert.equal(result.exactMatch, true);
  assert.equal(result.serializedDiffers, false);
});

test("decset_1048_restore_cursor is an exact serialize match for save/restore outside alt-screen", async () => {
  const result = await compareFixture("decset_1048_restore_cursor");

  assert.equal(result.fixture, "decset_1048_restore_cursor");
  assert.equal(result.semanticMatch, true);
  assert.equal(result.exactMatch, true);
  assert.equal(result.serializedDiffers, false);
});

test("decset_1048_restore_hidden_cursor matches xterm's saved visibility behavior", async () => {
  const result = await compareFixture("decset_1048_restore_hidden_cursor");

  assert.equal(result.fixture, "decset_1048_restore_hidden_cursor");
  assert.equal(result.semanticMatch, true);
  assert.equal(result.exactMatch, true);
  assert.equal(result.serializedDiffers, false);
});

test("insert_mode_active is an exact serialize match with ANSI insert mode enabled", async () => {
  const result = await compareFixture("insert_mode_active");

  assert.equal(result.fixture, "insert_mode_active");
  assert.equal(result.semanticMatch, true);
  assert.equal(result.exactMatch, true);
  assert.equal(result.serializedDiffers, false);
});

test("application_cursor_keys_active is an exact serialize match with DECCKM enabled", async () => {
  const result = await compareFixture("application_cursor_keys_active");

  assert.equal(result.fixture, "application_cursor_keys_active");
  assert.equal(result.semanticMatch, true);
  assert.equal(result.exactMatch, true);
  assert.equal(result.serializedDiffers, false);
});

test("application_keypad_active is an exact serialize match with keypad application mode enabled", async () => {
  const result = await compareFixture("application_keypad_active");

  assert.equal(result.fixture, "application_keypad_active");
  assert.equal(result.semanticMatch, true);
  assert.equal(result.exactMatch, true);
  assert.equal(result.serializedDiffers, false);
});

test("normal_mouse_active is an exact serialize match with normal mouse tracking enabled", async () => {
  const result = await compareFixture("normal_mouse_active");

  assert.equal(result.fixture, "normal_mouse_active");
  assert.equal(result.semanticMatch, true);
  assert.equal(result.exactMatch, true);
  assert.equal(result.serializedDiffers, false);
});

test("button_mouse_active is an exact serialize match with button-event mouse tracking enabled", async () => {
  const result = await compareFixture("button_mouse_active");

  assert.equal(result.fixture, "button_mouse_active");
  assert.equal(result.semanticMatch, true);
  assert.equal(result.exactMatch, true);
  assert.equal(result.serializedDiffers, false);
});

test("any_mouse_active is an exact serialize match with any-event mouse tracking enabled", async () => {
  const result = await compareFixture("any_mouse_active");

  assert.equal(result.fixture, "any_mouse_active");
  assert.equal(result.semanticMatch, true);
  assert.equal(result.exactMatch, true);
  assert.equal(result.serializedDiffers, false);
});

test("focus_plus_normal_mouse is an exact serialize match for combined focus and mouse modes", async () => {
  const result = await compareFixture("focus_plus_normal_mouse");

  assert.equal(result.fixture, "focus_plus_normal_mouse");
  assert.equal(result.semanticMatch, true);
  assert.equal(result.exactMatch, true);
  assert.equal(result.serializedDiffers, false);
});

test("paste_focus_mouse is an exact serialize match for bracketed paste, focus, and mouse ordering", async () => {
  const result = await compareFixture("paste_focus_mouse");

  assert.equal(result.fixture, "paste_focus_mouse");
  assert.equal(result.semanticMatch, true);
  assert.equal(result.exactMatch, true);
  assert.equal(result.serializedDiffers, false);
});

test("insert_focus_mouse is an exact serialize match for insert plus focus and mouse ordering", async () => {
  const result = await compareFixture("insert_focus_mouse");

  assert.equal(result.fixture, "insert_focus_mouse");
  assert.equal(result.semanticMatch, true);
  assert.equal(result.exactMatch, true);
  assert.equal(result.serializedDiffers, false);
});

test("normal_plus_sgr_mouse matches xterm's ignored SGR mouse encoding state", async () => {
  const result = await compareFixture("normal_plus_sgr_mouse");

  assert.equal(result.fixture, "normal_plus_sgr_mouse");
  assert.equal(result.semanticMatch, true);
  assert.equal(result.exactMatch, true);
  assert.equal(result.serializedDiffers, false);
});

test("button_plus_sgr_mouse matches xterm's ignored SGR mouse encoding state", async () => {
  const result = await compareFixture("button_plus_sgr_mouse");

  assert.equal(result.fixture, "button_plus_sgr_mouse");
  assert.equal(result.semanticMatch, true);
  assert.equal(result.exactMatch, true);
  assert.equal(result.serializedDiffers, false);
});

test("focus_plus_button_mouse is an exact serialize match for focus and button-event mouse ordering", async () => {
  const result = await compareFixture("focus_plus_button_mouse");

  assert.equal(result.fixture, "focus_plus_button_mouse");
  assert.equal(result.semanticMatch, true);
  assert.equal(result.exactMatch, true);
  assert.equal(result.serializedDiffers, false);
});

test("less_small_file_open is an exact serialize match for a real less startup screen", async () => {
  const result = await compareFixture("less_small_file_open");

  assert.equal(result.fixture, "less_small_file_open");
  assert.equal(result.semanticMatch, true);
  assert.equal(result.exactMatch, true);
  assert.equal(result.serializedDiffers, false);
});

test("less_page_forward is an exact serialize match for a real less page-forward screen", async () => {
  const result = await compareFixture("less_page_forward");

  assert.equal(result.fixture, "less_page_forward");
  assert.equal(result.semanticMatch, true);
  assert.equal(result.exactMatch, true);
  assert.equal(result.serializedDiffers, false);
});

test("vim_small_file_open is an exact serialize match for a real vim startup screen", async () => {
  const result = await compareFixture("vim_small_file_open");

  assert.equal(result.fixture, "vim_small_file_open");
  assert.equal(result.semanticMatch, true);
  assert.equal(result.exactMatch, true);
  assert.equal(result.serializedDiffers, false);
});

test("vim_percent_sgr_probe isolates Vim's CSI percent-SGR behavior", async () => {
  const result = await compareFixture("vim_percent_sgr_probe");

  assert.equal(result.fixture, "vim_percent_sgr_probe");
  assert.equal(result.semanticMatch, true);
  assert.equal(result.exactMatch, true);
  assert.equal(result.serializedDiffers, false);
});

test("alt_screen_late_blue_fill is an exact serialize match for xterm's leading color hoist behavior", async () => {
  const result = await compareFixture("alt_screen_late_blue_fill");

  assert.equal(result.fixture, "alt_screen_late_blue_fill");
  assert.equal(result.semanticMatch, true);
  assert.equal(result.exactMatch, true);
  assert.equal(result.serializedDiffers, false);
});

test("vim_cursor_down is an exact serialize match for a real vim cursor movement", async () => {
  const result = await compareFixture("vim_cursor_down");

  assert.equal(result.fixture, "vim_cursor_down");
  assert.equal(result.semanticMatch, true);
  assert.equal(result.exactMatch, true);
  assert.equal(result.serializedDiffers, false);
});

test("vim_insert_escape is an exact serialize match for a real vim insert-and-escape sequence", async () => {
  const result = await compareFixture("vim_insert_escape");

  assert.equal(result.fixture, "vim_insert_escape");
  assert.equal(result.semanticMatch, true);
  assert.equal(result.exactMatch, true);
  assert.equal(result.serializedDiffers, false);
});

test("nvim_small_file_open is an exact serialize match for a real nvim startup screen", async () => {
  const result = await compareFixture("nvim_small_file_open");

  assert.equal(result.fixture, "nvim_small_file_open");
  assert.equal(result.semanticMatch, true);
  assert.equal(result.exactMatch, true);
  assert.equal(result.serializedDiffers, false);
});

test("nvim_insert_escape is an exact serialize match for a real nvim insert-and-escape sequence", async () => {
  const result = await compareFixture("nvim_insert_escape");

  assert.equal(result.fixture, "nvim_insert_escape");
  assert.equal(result.semanticMatch, true);
  assert.equal(result.exactMatch, true);
  assert.equal(result.serializedDiffers, false);
});

test("nvim_cursor_down is an exact serialize match for a real nvim cursor movement", async () => {
  const result = await compareFixture("nvim_cursor_down");

  assert.equal(result.fixture, "nvim_cursor_down");
  assert.equal(result.semanticMatch, true);
  assert.equal(result.exactMatch, true);
  assert.equal(result.serializedDiffers, false);
});
