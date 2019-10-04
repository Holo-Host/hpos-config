use holo_config::{Config, Seed, SeedData};

use docopt::Docopt;
use failure::Error;
use serde::*;
use sha2::{Sha256, Digest};
use std::{env, fs::File, io, path::PathBuf};
use url::Url;

const USAGE: &'static str = "
Usage: holo-config-generate --email EMAIL --password STRING [--seed-from PATH] [--encrypt]
       holo-config-generate --help

Creates Holo config file that contains seed and admin email/password.

Options:
  --email EMAIL      HoloPort admin email address
  --password STRING  HoloPort admin password
  --seed-from PATH   Use SHA-512 hash of given file truncated to 256 bits as seed
  --encrypt          Encrypt the seed using the admin password
";

#[derive(Deserialize)]
struct Args {
    flag_email: String,
    flag_password: String,
    flag_seed_from: Option<PathBuf>,
    flag_encrypt: bool,
}

pub fn to_url(hcid: &str) -> Result<Url, Error> {
    let url = Url::parse(&format!("https://{}.holohost.net", hcid))?;
    Ok(url)
}

fn main() -> Result<(), Error> {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.argv(env::args().into_iter()).deserialize())
        .unwrap_or_else(|e| e.exit());

    let maybe_seed = match args.flag_seed_from {
        None => None,
        Some(path) => {
            let mut hasher = Sha256::new();
            let mut file = File::open(path)?;
            io::copy(&mut file, &mut hasher)?;
            let entropy = hasher.result();
            Some(Seed::from_bytes(&entropy[..])?)
        }
    };
    let (config, hcid) = Config::new(
        args.flag_email, args.flag_password, maybe_seed, args.flag_encrypt)?;
    eprintln!("{}", to_url(&hcid.to_string())?);
    println!("{}", serde_json::to_string_pretty(&config)?);

    Ok(())
}
