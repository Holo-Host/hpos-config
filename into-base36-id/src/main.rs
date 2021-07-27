use ed25519_dalek::*;
use failure::*;
use hpos_config_core::*;

use std::io::stdin;

fn main() -> Fallible<()> {
    match serde_json::from_reader(stdin())? {
        Config::V1 { seed, .. } => {
            let secret_key = SecretKey::from_bytes(&seed)?;
            let public_key = PublicKey::from(&secret_key);
            println!("{}", public_key::to_base36_id(&public_key));
        }
        Config::V2 { seed, .. } => {
            let secret_key = SecretKey::from_bytes(&seed)?;
            let public_key = PublicKey::from(&secret_key);
            println!("{}", public_key::to_base36_id(&public_key));
        }
    }

    Ok(())
}
