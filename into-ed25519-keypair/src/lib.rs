use ed25519_dalek::*;
use failure::*;
use hpos_config_core::{config::Seed, Config};

// TODO get pub key for the device bundle
pub fn holoport_public_key(config: Config) -> Result<PublicKey, Error> {
    match config {
        Config::V1 { seed, .. } => {
            let secret_key = SecretKey::from_bytes(&seed)?;
            Ok(PublicKey::from(&secret_key))
        }
        Config::V2 { holoport_id, .. } => Ok(holoport_id.to_owned()),
    }
}

pub fn encoded_ed25519_keypair(config: Config) -> Result<String, Error> {
    match config {
        Config::V1 { seed, .. } => {
            let secret_key = SecretKey::from_bytes(&seed)?;
            Ok(encrypt_key(&seed, PublicKey::from(&secret_key)))
        }
        Config::V2 { .. } => Ok("TODO".to_string()),
    }
}

pub fn decoded_to_ed25519_keypair(blob: &String) -> Result<Keypair, Error> {
    let decoded_key = base64::decode(blob)?;
    Ok(Keypair {
        public: PublicKey::from_bytes(&decoded_key[32..64].to_vec())?,
        secret: SecretKey::from_bytes(&decoded_key[64..].to_vec())?,
    })
}

fn encrypt_key(seed: &Seed, public_key: PublicKey) -> String {
    // For now lair does not take in any encrypted bytes so we pass back an empty encrypted byte string
    let mut encrypted_key = vec![
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0,
    ];
    encrypted_key.extend(&public_key.to_bytes());
    encrypted_key.extend(seed.to_vec());
    base64::encode(&encrypted_key)
}
