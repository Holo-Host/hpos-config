use hpos_config_core::{config::Seed, public_key, Config};

use docopt::Docopt;
use ed25519_dalek::*;
use failure::Error;
use rand::Rng;
use serde::*;
use sha2::{Digest, Sha512Trunc256};
use std::{env, fs::File, io, path::PathBuf};

const USAGE: &str = "
Usage: hpos-config-gen-cli --email EMAIL --password STRING --registration-code STRING --derivation-path STRING --device-bundle STRING [--seed-from PATH]
       hpos-config-gen-cli --help

Creates HoloPortOS config file that contains seed and admin email/password.

Options:
  --email EMAIL                 HoloPort admin email address
  --password STRING             HoloPort admin password
  --registration-code CODE      HoloPort admin password
  --derivation-path STRING      Derivation path of the seed
  --device-bundle STRING        Device Bundle
  --seed-from PATH              Use SHA-512 hash of given file truncated to 256 bits as seed
";

#[derive(Deserialize)]
struct Args {
    flag_email: String,
    flag_password: String,
    flag_registration_code: String,
    flag_derivation_path: String,
    flag_device_bundle: String,
    flag_seed_from: Option<PathBuf>,
}

fn main() -> Result<(), Error> {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.argv(env::args()).deserialize())
        .unwrap_or_else(|e| e.exit());

    let seed = match args.flag_seed_from {
        None => rand::thread_rng().gen::<Seed>(),
        Some(path) => {
            let mut hasher = Sha512Trunc256::new();
            let mut file = File::open(path)?;
            io::copy(&mut file, &mut hasher)?;

            let seed: Seed = hasher.result().into();
            seed
        }
    };

    let secret_key = SigningKey::from_bytes(&seed);

    let (config, public_key) = Config::new_v2(
        args.flag_email,
        args.flag_password,
        args.flag_registration_code,
        args.flag_derivation_path,
        args.flag_device_bundle,
        VerifyingKey::from(&secret_key),
    )?;
    eprintln!("{}", public_key::to_url(&public_key)?);
    println!("{}", serde_json::to_string_pretty(&config)?);
    Ok(())
}
