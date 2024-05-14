use ed25519_dalek::{ed25519, Keypair, PublicKey, SecretKey};
use hc_seed_bundle::*;
use hpos_config_core::Config;

/// get pub key for the device bundle in the config
pub async fn holoport_public_key(
    config: &Config,
    passphrase: Option<String>,
) -> SeedExplorerResult<PublicKey> {
    match config {
        Config::V1 { seed, .. } => {
            let secret_key = SecretKey::from_bytes(seed)?;
            Ok(PublicKey::from(&secret_key))
        }
        Config::V2 { device_bundle, .. } => {
            /*
                decode base64 string to locked device bundle
                password is pass for now
                unlock it and get the signPubKey
            */
            let Keypair { public, .. } = unlock(device_bundle, passphrase).await?;
            Ok(public)
        }
        Config::V3 { holoport_id, .. } => Ok(holoport_id.to_owned()),
    }
}

/// get key for the device bundle in the config
pub async fn holoport_key(
    config: &Config,
    passphrase: Option<String>,
) -> SeedExplorerResult<Keypair> {
    match config {
        Config::V1 { seed, .. } => {
            let secret = SecretKey::from_bytes(seed)?;
            Ok(Keypair {
                public: PublicKey::from(&secret),
                secret,
            })
        }
        Config::V2 { device_bundle, .. } | Config::V3 { device_bundle, .. } => {
            /*
                decode base64 string to locked device bundle
                password is pass for now
                unlock it and get the signPubKey
            */
            unlock(device_bundle, passphrase).await
        }
    }
}

/// encode the ed25519 keypair making it compatible with lair (<v0.0.6)
pub async fn encoded_ed25519_keypair(
    config: &Config,
    passphrase: Option<String>,
) -> SeedExplorerResult<String> {
    match config {
        Config::V1 { seed, .. } => {
            let secret_key = SecretKey::from_bytes(seed)?;
            Ok(encrypt_key(&secret_key, &PublicKey::from(&secret_key)))
        }
        Config::V2 { device_bundle, .. } | Config::V3 { device_bundle, .. } => {
            /*
                decode base64 string to locked device bundle
                password is pass for now
                unlock it and get the signPubKey
                Pass the Seed and PublicKey into `encrypt_key(seed, pubKey)`
            */
            let Keypair { public, secret } = unlock(device_bundle, passphrase).await?;
            Ok(encrypt_key(&secret, &public))
        }
    }
}

/// decode the ed25519 keypair making it compatible with lair (<v0.0.6)
pub fn decoded_to_ed25519_keypair(blob: &String) -> SeedExplorerResult<Keypair> {
    let decoded_key = base64::decode(blob)?;
    Ok(Keypair {
        public: PublicKey::from_bytes(&decoded_key[32..64].to_vec())?,
        secret: SecretKey::from_bytes(&decoded_key[64..].to_vec())?,
    })
}

// todo: we need to update this for production
/// For now lair does not take in any encrypted bytes so we pass back an empty encrypted byte string
pub fn encrypt_key(seed: &SecretKey, public_key: &PublicKey) -> String {
    let mut encrypted_key = vec![
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0,
    ];
    encrypted_key.extend(&public_key.to_bytes());
    encrypted_key.extend(seed.to_bytes());
    base64::encode(&encrypted_key)
}

/// unlock seed_bundles to access the pub-key and seed
pub async fn unlock(
    device_bundle: &String,
    passphrase: Option<String>,
) -> SeedExplorerResult<Keypair> {
    let cipher = base64::decode_config(device_bundle, base64::URL_SAFE_NO_PAD)?;
    match UnlockedSeedBundle::from_locked(&cipher).await?.remove(0) {
        LockedSeedCipher::PwHash(cipher) => {
            let passphrase = passphrase
                .as_ref()
                .ok_or(SeedExplorerError::PasswordRequired)?;
            let passphrase = sodoken::BufRead::from(passphrase.as_bytes().to_vec());
            let seed = cipher.unlock(passphrase).await?;
            Ok(Keypair {
                public: PublicKey::from_bytes(&*seed.get_sign_pub_key().read_lock())?,
                secret: SecretKey::from_bytes(&*seed.get_seed().read_lock())?,
            })
        }
        _ => Err(SeedExplorerError::UnsupportedCipher),
    }
}

#[derive(thiserror::Error, Debug)]
pub enum SeedExplorerError {
    #[error(transparent)]
    OneErr(#[from] hc_seed_bundle::dependencies::one_err::OneErr),
    #[error(transparent)]
    Ed25519Error(#[from] ed25519::Error),
    #[error(transparent)]
    DecodeError(#[from] base64::DecodeError),
    #[error("Seed hash unsupported cipher type")]
    UnsupportedCipher,
    #[error("Password required to unlock seed")]
    PasswordRequired,
    #[error("Generic Error: {0}")]
    Generic(String),
}

pub type SeedExplorerResult<T> = Result<T, SeedExplorerError>;
