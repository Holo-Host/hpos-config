use failure::*;
use hpos_config_core::*;
use std::io::stdin;

fn main() -> Fallible<()> {
    match serde_json::from_reader(stdin())? {
        Config::V1 { .. } => Ok(()),
        Config::V2 { .. } => Ok(()),
    }
}
