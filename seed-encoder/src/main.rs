//! This binary is used for generating a encoded key from the
//! seed, this is used by the `--load_ed25519_keypair_from_seed` in lair
//!

use ed25519_dalek::*;
use failure::*;
use hpos_config_core::*;
use hpos_config_seed_bundle_explorer::{encrypt_key, unlock};
use std::path::PathBuf;
use structopt::StructOpt;

#[tokio::main]
async fn main() -> Fallible<()> {
    #[derive(StructOpt)]
    struct Cli {
        #[structopt(long = "config-path")]
        /// The path to the hpos-config file
        config_path: PathBuf,
        #[structopt(long = "password")]
        /// The password to unlock the device-bundle
        password: String,
    }

    let Cli {
        config_path,
        password,
    } = Cli::from_args();
    use std::fs::File;
    let config_file = File::open(config_path).context("failed to open file")?;
    match serde_json::from_reader(config_file)? {
        Config::V1 { seed, .. } => {
            let secret_key = SecretKey::from_bytes(&seed)?;
            let public_key = PublicKey::from(&secret_key);
            println!("{}", encrypt_key(&secret_key, &public_key));
        }
        Config::V2 { device_bundle, .. } => {
            // take in password
            let Keypair { public, secret } = unlock(&device_bundle, Some(password)).await.unwrap();
            println!("{}", encrypt_key(&secret, &public));
        }
    }

    Ok(())
}