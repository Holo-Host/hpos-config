use holo_config::{keystore, public_key, Config};

use failure::Error;
use std::io;

fn main() -> Result<(), Error> {
    let Config::V1 { seed, .. } = serde_json::from_reader(io::stdin())?;

    let (keystore, public_key) = keystore::from_seed(&seed)?;
    println!("{}", serde_json::to_string_pretty(&keystore)?);
    println!("{}", public_key::to_hcid(&public_key)?);

    Ok(())
}
