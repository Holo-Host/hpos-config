use holo_config_core::{config::Seed, public_key, Config};

use ed25519_dalek::PublicKey;
use failure::Error;
use std::io;

use holochain_common::DEFAULT_PASSPHRASE;
use holochain_conductor_api::key_loaders::mock_passphrase_manager;
use holochain_conductor_api::keystore::*;
use holochain_dpki::CODEC_HCS0;

pub fn keystore_from_seed(seed: &Seed) -> Result<(Keystore, PublicKey), Error> {
    let passphrase_manager = mock_passphrase_manager(DEFAULT_PASSPHRASE.into());
    let mut keystore = Keystore::new(passphrase_manager, None)?;

    keystore.add_seed(STANDALONE_ROOT_SEED, seed)?;
    let (public_key_hcid, _) =
        keystore.add_keybundle_from_seed(STANDALONE_ROOT_SEED, PRIMARY_KEYBUNDLE_ID)?;
    let public_key_bytes = CODEC_HCS0.decode(&public_key_hcid)?;
    let public_key = PublicKey::from_bytes(&public_key_bytes)?;

    Ok((keystore, public_key))
}

fn main() -> Result<(), Error> {
    let Config::V1 { seed, .. } = serde_json::from_reader(io::stdin())?;

    let (keystore, public_key) = keystore_from_seed(&seed)?;
    eprintln!("{}", public_key::to_hcid(&public_key)?);
    println!("{}", serde_json::to_string_pretty(&keystore)?);

    Ok(())
}
