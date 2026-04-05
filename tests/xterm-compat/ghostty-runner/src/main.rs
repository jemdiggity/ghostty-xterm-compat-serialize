use ghostty_xterm_compat_serialize::run_fixture_by_name;
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let fixture_name = env::args().nth(1).ok_or("missing fixture name")?;
    let output = run_fixture_by_name(&fixture_name)?;
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}
