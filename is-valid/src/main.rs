use failure::*;
use hpos_config_core::*;
use std::io::stdin;

fn main() -> Fallible<()> {
    let Config::V1 { .. } = serde_json::from_reader(stdin())?;
    Ok(())
}
