use serde_json::Value;
use std::process::Command;

#[test]
fn startup_prompt_fixture_produces_json_metadata() {
    let output = Command::new(env!("CARGO_BIN_EXE_ghostty-xterm-compat-runner"))
        .arg("startup_prompt")
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("failed to run ghostty runner");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let value: Value = serde_json::from_str(stdout.trim()).expect("stdout should be valid JSON");

    assert_eq!(value["fixture"], "startup_prompt");
    let serialized = value["serializedCandidate"]
        .as_str()
        .expect("serializedCandidate should be a string");
    assert!(
        serialized.contains("OpenAI Codex"),
        "serialized candidate should contain the prompt frame: {serialized}"
    );
    assert!(value["cursorX"].as_u64().unwrap_or(0) > 0);
    assert!(value["cursorY"].as_u64().unwrap_or(0) > 0);
}

#[test]
fn prompt_redraw_fixture_matches_xterm_reference_shape() {
    let output = Command::new(env!("CARGO_BIN_EXE_ghostty-xterm-compat-runner"))
        .arg("prompt_redraw")
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("failed to run ghostty runner");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let value: Value = serde_json::from_str(stdout.trim()).expect("stdout should be valid JSON");
    let serialized = value["serializedCandidate"]
        .as_str()
        .expect("serializedCandidate should be a string");

    assert!(
        serialized.contains("\u{1b}[1C\u{1b}[48;2;57;57;57m\u{1b}[79X"),
        "serialized candidate should include xterm-style fill-row output: {serialized:?}"
    );
    assert!(
        serialized.contains(
            "Pizza is good. If you want, I can help with dough, toppings, or a place to order."
        ),
        "serialized candidate should keep the wrapped sentence intact: {serialized:?}"
    );
}
