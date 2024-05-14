use ed25519_dalek::{ed25519, SigningKey, VerifyingKey};
use hc_seed_bundle::*;
use hpos_config_core::Config;

/// get pub key for the device bundle in the config
pub async fn holoport_public_key(
    config: &Config,
    passphrase: Option<String>,
) -> SeedExplorerResult<VerifyingKey> {
    match config {
        Config::V1 { seed, .. } => {
            let secret_key = SigningKey::from_bytes(seed);
            Ok(VerifyingKey::from(&secret_key))
        }
        Config::V2 { device_bundle, .. } => {
            /*
                decode base64 string to locked device bundle
                password is pass for now
                unlock it and get the signPubKey
            */
            let secret = unlock(device_bundle, passphrase).await?;
            Ok(secret.verifying_key())
        }
        Config::V3 { holoport_id, .. } => Ok(holoport_id.to_owned()),
    }
}

/// get key for the device bundle in the config
pub async fn holoport_key(
    config: &Config,
    passphrase: Option<String>,
) -> SeedExplorerResult<SigningKey> {
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
            let secret_key = SigningKey::from_bytes(seed);
            Ok(encrypt_key(&secret_key, &VerifyingKey::from(&secret_key)))
        }
        Config::V2 { device_bundle, .. } | Config::V3 { device_bundle, .. } => {
            /*
                decode base64 string to locked device bundle
                password is pass for now
                unlock it and get the signPubKey
                Pass the Seed and PublicKey into `encrypt_key(seed, pubKey)`
            */
            let secret = unlock(device_bundle, passphrase).await?;
            Ok(encrypt_key(&secret, &secret.verifying_key()))
        }
    }
}

/// decode the ed25519 keypair making it compatible with lair (<v0.0.6)
pub fn decoded_to_ed25519_keypair(blob: &String) -> SeedExplorerResult<SigningKey> {
    let decoded_key = base64::decode(blob)?;

    let decoded_key_bytes: [u8; 32] = match decoded_key[64..].try_into() {
        Ok(b) => b,
        Err(_) => {
            return Err(SeedExplorerError::Generic(
                "Unable to extract private key starting at position 64".into(),
            ))
        }
    };

    Ok(SigningKey::from_bytes(&decoded_key_bytes))
}

// todo: we need to update this for production
/// For now lair does not take in any encrypted bytes so we pass back an empty encrypted byte string
pub fn encrypt_key(seed: &SigningKey, public_key: &VerifyingKey) -> String {
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
) -> SeedExplorerResult<SigningKey> {
    let cipher = base64::decode_config(device_bundle, base64::URL_SAFE_NO_PAD)?;
    match UnlockedSeedBundle::from_locked(&cipher).await?.remove(0) {
        LockedSeedCipher::PwHash(cipher) => {
            let passphrase = passphrase
                .as_ref()
                .ok_or(SeedExplorerError::PasswordRequired)?;
            let passphrase = sodoken::BufRead::from(passphrase.as_bytes().to_vec());
            let seed = cipher.unlock(passphrase).await?;

            let seed_bytes: [u8; 32] = match (&*seed.get_seed().read_lock())[0..32].try_into() {
                Ok(b) => b,
                Err(_) => {
                    return Err(SeedExplorerError::Generic(
                        "Seed buffer is not 32 bytes long".into(),
                    ))
                }
            };

            Ok(SigningKey::from_bytes(&seed_bytes))
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
