use failure::Error;
use std::io::stdin;

use holo_config_core::{config::Seed, Config};

use holochain_common::DEFAULT_PASSPHRASE;
use holochain_conductor_api::key_loaders::mock_passphrase_manager;
use holochain_conductor_api::keystore::*;

pub fn keystore_from_seed(seed: &Seed) -> Result<(Keystore, String), Error> {
    let passphrase_manager = mock_passphrase_manager(DEFAULT_PASSPHRASE.into());
    let mut keystore = Keystore::new(passphrase_manager, None)?;

    keystore.add_seed(STANDALONE_ROOT_SEED, seed)?;
    let (public_key_hcid, _) =
        keystore.add_keybundle_from_seed(STANDALONE_ROOT_SEED, PRIMARY_KEYBUNDLE_ID)?;

    Ok((keystore, public_key_hcid))
}

fn main() -> Result<(), Error> {
    let Config::V1 { seed, .. } = serde_json::from_reader(stdin())?;
    let (keystore, public_key_hcid) = keystore_from_seed(&seed)?;

    eprintln!("{}", public_key_hcid);
    println!("{}", serde_json::to_string_pretty(&keystore)?);

    Ok(())
}
