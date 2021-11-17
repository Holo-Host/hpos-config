use ed25519_dalek::{Keypair, PublicKey, SecretKey};
use failure::*;
use hc_seed_bundle::*;
use hpos_config_core::Config;

/// get pub key for the device bundle in the config
pub async fn holoport_public_key(
    config: &Config,
    passphrase: Option<String>,
) -> Result<PublicKey, Error> {
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
            let Keypair { public, .. } = unlock(device_bundle, passphrase).await.unwrap();
            Ok(public)
        }
    }
}

/// encode the ed25519 keypair making it compatible with lair (<v0.0.6)
pub async fn encoded_ed25519_keypair(
    config: &Config,
    passphrase: Option<String>,
) -> Result<String, Error> {
    match config {
        Config::V1 { seed, .. } => {
            let secret_key = SecretKey::from_bytes(seed)?;
            Ok(encrypt_key(&secret_key, &PublicKey::from(&secret_key)))
        }
        Config::V2 { device_bundle, .. } => {
            /*
                decode base64 string to locked device bundle
                password is pass for now
                unlock it and get the signPubKey
                Pass the Seed and PublicKey into `encrypt_key(seed, pubKey)`
            */
            let Keypair { public, secret } = unlock(device_bundle, passphrase).await.unwrap();
            Ok(encrypt_key(&secret, &public))
        }
    }
}

/// decode the ed25519 keypair making it compatible with lair (<v0.0.6)
pub fn decoded_to_ed25519_keypair(blob: &String) -> Result<Keypair, Error> {
    let decoded_key = base64::decode(blob)?;
    Ok(Keypair {
        public: PublicKey::from_bytes(&decoded_key[32..64].to_vec())?,
        secret: SecretKey::from_bytes(&decoded_key[64..].to_vec())?,
    })
}

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
pub async fn unlock(device_bundle: &String, passphrase: Option<String>) -> Result<Keypair, String> {
    let cipher = base64::decode_config(device_bundle, base64::URL_SAFE_NO_PAD).unwrap();
    match UnlockedSeedBundle::from_locked(&cipher)
        .await
        .unwrap()
        .remove(0)
    {
        LockedSeedCipher::PwHash(cipher) => {
            let passphrase = passphrase.as_ref().unwrap();
            let passphrase = sodoken::BufRead::from(passphrase.as_bytes().to_vec());
            let seed = cipher.unlock(passphrase).await.unwrap();
            Ok(Keypair {
                public: PublicKey::from_bytes(&*seed.get_sign_pub_key().read_lock()).unwrap(),
                secret: SecretKey::from_bytes(&*seed.get_seed().read_lock()).unwrap(),
            })
        }
        _ => Err("unsupported cipher".to_string()),
    }
}
