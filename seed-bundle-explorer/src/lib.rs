use ed25519_dalek::{Keypair, PublicKey, SecretKey};
use failure::*;
use hc_seed_bundle::{LockedSeedCipher, UnlockedSeedBundle};
use hpos_config_core::Config;

/// get pub key for the device bundle in the config
pub async fn holoport_public_key(config: Config, passphrase: String) -> Result<PublicKey, Error> {
    match config {
        Config::V1 { seed, .. } => {
            let secret_key = SecretKey::from_bytes(&seed)?;
            Ok(PublicKey::from(&secret_key))
        }
        Config::V2 { device_bundle, .. } => {
            /*
                decode base64 string to locked device bundle
                password is pass for now
                unlock it and get the signPubKey
            */
            let (_, pub_key) = unlock(device_bundle, passphrase).await.unwrap();
            Ok(pub_key)
        }
    }
}

/// encode the ed25519 keypair making it compatible with lair (<v0.0.6)
pub async fn encoded_ed25519_keypair(config: Config, passphrase: String) -> Result<String, Error> {
    match config {
        Config::V1 { seed, .. } => {
            let secret_key = SecretKey::from_bytes(&seed)?;
            Ok(encrypt_key(&secret_key, &PublicKey::from(&secret_key)))
        }
        Config::V2 { device_bundle, .. } => {
            /*
                decode base64 string to locked device bundle
                password is pass for now
                unlock it and get the signPubKey
                Pass the Seed and PublicKey into `encrypt_key(seed, pubKey)`
            */
            let (seed, pub_key) = unlock(device_bundle, passphrase).await.unwrap();
            Ok(encrypt_key(&seed, &pub_key))
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
fn encrypt_key(seed: &SecretKey, public_key: &PublicKey) -> String {
    let mut encrypted_key = vec![
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0,
    ];
    encrypted_key.extend(&public_key.to_bytes());
    encrypted_key.extend(seed.to_bytes());
    base64::encode(&encrypted_key)
}

/// unlock seed_bundles to access the pub-key and seed
async fn unlock(
    device_bundle: String,
    passphrase: String,
) -> Result<(SecretKey, PublicKey), String> {
    let cipher = base64::decode(&device_bundle).unwrap();
    match UnlockedSeedBundle::from_locked(&cipher)
        .await
        .unwrap()
        .remove(0)
    {
        LockedSeedCipher::PwHash(cipher) => {
            let passphrase = sodoken::BufRead::from(passphrase.as_bytes().to_vec());
            let seed = cipher.unlock(passphrase).await.unwrap();
            return Ok((
                SecretKey::from_bytes(&*seed.get_seed().read_lock()).unwrap(),
                PublicKey::from_bytes(&*seed.get_sign_pub_key().read_lock()).unwrap(),
            ));
        }
        _ => return Err("unsupported cipher".to_string()),
    }
}
