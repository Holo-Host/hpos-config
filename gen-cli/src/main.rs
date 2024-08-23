use hpos_config_core::{config::Seed, public_key, Config};

use clap::Parser;
use ed25519_dalek::*;
use failure::Error;
use rand::Rng;
use sha2::{Digest, Sha512Trunc256};
use std::{fs::File, io};

#[derive(Parser, Clone)]
#[command(about = "Creates HoloPortOS config file that contains seed and admin email/password.")]
struct ClapArgs {
    #[arg(
        long,
        value_parser,
        value_name = "EMAIL",
        help = "HoloPort admin email address"
    )]
    email: String,
    #[arg(
        long,
        value_parser,
        value_name = "PASSWORD",
        help = "HoloPort admin password"
    )]
    password: String,
    #[arg(
        long,
        value_parser,
        value_name = "CODE",
        help = "HoloPort registration code"
    )]
    registration_code: String,
    #[arg(long, value_parser, value_name = "STRING", help = "Revocation key")]
    revocation_key: Option<String>,
    #[arg(
        long,
        value_parser,
        value_name = "PATH",
        help = "Derivation path of the seed"
    )]
    derivation_path: String,
    #[arg(
        long,
        value_parser,
        value_name = "STRING",
        help = "HoloPort Device bundle"
    )]
    device_bundle: String,
    #[arg(
        long,
        value_parser,
        value_name = "PATH",
        help = "Use SHA-512 hash of given file, truncated to 256 bits, as seed"
    )]
    seed_from: Option<String>,
}

fn main() -> Result<(), Error> {
    let args = ClapArgs::parse();

    let seed = match args.seed_from {
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
    let revocation_key = match &args.revocation_key {
        None => VerifyingKey::from(&secret_key),
        Some(rk) => {
            let public_key_bytes: &[u8; PUBLIC_KEY_LENGTH] = rk.as_bytes().try_into()?;
            VerifyingKey::from_bytes(public_key_bytes)?
        }
    };

    let (config, public_key) = Config::new(
        args.email,
        args.password,
        args.registration_code,
        revocation_key,
        args.derivation_path,
        args.device_bundle,
        VerifyingKey::from(&secret_key),
    )?;
    eprintln!("{}", public_key::to_url(&public_key)?);
    println!("{}", serde_json::to_string_pretty(&config)?);
    Ok(())
}
