use ed25519_dalek::*;
use failure::*;
use hpos_config::*;

use std::io::stdin;

fn main() -> Fallible<()> {
    let Config::V1 { seed, .. } = serde_json::from_reader(stdin())?;
    let secret_key = SecretKey::from_bytes(&seed)?;
    let public_key = PublicKey::from(&secret_key);

    println!("{}", public_key::to_base36_id(&public_key));

    Ok(())
}
