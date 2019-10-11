use holo_config::{Config, ConfigSeed};

use docopt::Docopt;
use failure::Error;
use serde::*;
use std::{env, io};

const USAGE: &'static str = "
Usage: holo-config-derive
       holo-config-derive --email EMAIL [--password STRING] [--decrypt]
       holo-config-derive --help

Reads a JSON Config from stdin containing Seed and public Agent and Admin keys.  Optionally (if
email and password provided), also derives the private Agent and Admin keys (decrypting the Seed, if
necessary to do so).  Finally, emits the Config in JSON form (encrypting the Seed, if it was
encrypted in the provided Config).

Options:
  --email EMAIL      HoloPort admin email address
  --password STRING  HoloPort admin password
  --decrypt          Emit the seed decrypted, instead of as in provided Config
";

#[derive(Deserialize)]
struct Args {
    flag_email: Option<String>,
    flag_password: Option<String>,
    flag_decrypt: bool,
}

fn main() -> Result<(), Error> {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.argv(env::args().into_iter()).deserialize())
        .unwrap_or_else(|e| e.exit());

    // Read the Config from stdin
    let config: Config = serde_json::from_reader(io::stdin())?;

    // If arguments were provided to encrypt or to derive keys, do so.
    if args.flag_email.is_some() {
        let password = args.flag_password.unwrap_or_else(|| {
            eprintln!("WARNING: Deriving HoloPort Config w/ empty passphrase!");
            "".to_string()
        });
        let Config::V1{ seed, .. } = config;
        let encrypt = if args.flag_decrypt { false } else { match seed {
            ConfigSeed::PlaintextSeed(_) => false,
            ConfigSeed::EncryptedSeed(_) => true,
        }};
        let config_result = Config::new(
            args.flag_email.unwrap(), password, Some(seed), encrypt)?;

        println!("{}", serde_json::to_string_pretty(&config_result)?);
    } else {
        // No key derivation or encryption desired; Just re-emit the parsed Config as-is.
        println!("{}", serde_json::to_string_pretty(&config)?);
    };

    Ok(())
}
