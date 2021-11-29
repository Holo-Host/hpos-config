use anyhow::{Context, Result};
use ed25519_dalek::*;
use hpos_config_core::*;
use hpos_config_seed_bundle_explorer::unlock;
use std::path::PathBuf;
use structopt::StructOpt;
use std::fs::File;

#[tokio::main]
async fn main() -> Result<()> {
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
        ..
    } = Cli::from_args();

    let config_file =
        File::open(&config_path).context(format!("failed to open file {}", &config_path.to_string_lossy()))?;
    match serde_json::from_reader(config_file)? {
        Config::V1 { seed, .. } => {
            let secret_key = SecretKey::from_bytes(&seed)?;
            let public_key = PublicKey::from(&secret_key);
            println!("{}", public_key::to_base36_id(&public_key));
        }
        Config::V2 { device_bundle, .. } => {
            // take in password
            let Keypair { public, .. } =
                unlock(&device_bundle, Some(password))
                    .await
                    .context(format!(
                        "unable to unlock the device bundle from {}",
                        &config_path.to_string_lossy()
                    ))?;
            println!("{}", public_key::to_base36_id(&public));
        }
    }

    Ok(())
}
