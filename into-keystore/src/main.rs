use docopt::Docopt;
use failure::Error;
use serde::*;
use std::env;
use std::fs::{self, File};
use std::path::PathBuf;

use holochain_common::DEFAULT_PASSPHRASE;
use holochain_conductor_api::key_loaders::mock_passphrase_manager;
use holochain_conductor_api::keystore::*;
use holochain_dpki::CODEC_HCS0;

use hpos_config_core::{config::Seed, Config};

const USAGE: &'static str = "
Usage: hpos-config-into-keystore --from <config_path> --to <keystore_path>
       hpos-config-into-keystore --help

Derives Holochain keystore along with public key files in Base36 and HCID format.

Options:
  --from PATH  Path to hpos-config.json
  --to PATH    Path to hpos-keystore.json
";

#[derive(Deserialize)]
struct Args {
    flag_from: PathBuf,
    flag_to: PathBuf,
}

pub fn keystore_from_seed(seed: &Seed) -> Result<(Keystore, String), Error> {
    let passphrase_manager = mock_passphrase_manager(DEFAULT_PASSPHRASE.into());
    let mut keystore = Keystore::new(passphrase_manager, None)?;

    keystore.add_seed(STANDALONE_ROOT_SEED, seed)?;
    let (public_key_hcid, _) =
        keystore.add_keybundle_from_seed(STANDALONE_ROOT_SEED, PRIMARY_KEYBUNDLE_ID)?;

    Ok((keystore, public_key_hcid))
}

fn main() -> Result<(), Error> {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.argv(env::args().into_iter()).deserialize())
        .unwrap_or_else(|e| e.exit());

    let config_path = File::open(args.flag_from)?;
    let Config::V1 { seed, .. } = serde_json::from_reader(config_path)?;
    let (keystore, public_key_hcid) = keystore_from_seed(&seed)?;

    let public_key = CODEC_HCS0.decode(&public_key_hcid)?;
    let public_key_base36 = base36::encode(&public_key);

    let keystore_path = args.flag_to;
    let base36_path = keystore_path.with_extension("base36.pub");
    let hcid_path = keystore_path.with_extension("hcid.pub");

    fs::write(base36_path, public_key_base36)?;
    fs::write(hcid_path, public_key_hcid)?;
    fs::write(keystore_path, serde_json::to_string(&keystore)?)?;

    Ok(())
}
