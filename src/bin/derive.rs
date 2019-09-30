use holo_config::{Config, keystore};

use failure::Error;

use std::fs::*;

fn main() -> Result<(), Error> {
    let config_str = read_to_string("holo-config.json")?;
    let Config::V1 { seed, .. } = serde_json::from_str(&config_str)?;

    let (keystore, public_key) = keystore::from_seed(&seed)?;
    println!("{}", serde_json::to_string_pretty(&keystore)?);

    Ok(())
}
