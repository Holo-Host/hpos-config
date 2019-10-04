
// DPKI-compatible pure-rust intrinsics for dealing in Seeds, Keypairs, password hashes
// 
// We'll use pure-rust cryptographic primatives, to ease WASM generation.  Later, we can transition
// to using Holochain's DPKI API.


use std::fmt;
use failure::Error;
use hcid;
use bip39::{
    Language, Mnemonic, MnemonicType
};
use sha2::{
    Sha512, Digest
};
use serde::{
    Deserialize, Deserializer, Serialize, Serializer, de
};

use ed25519_dalek::{
    PublicKey, SecretKey, Keypair, SignatureError,
};

pub const PUBLIC_KEY_SIZE: usize = 256/8;
pub const SECRET_KEY_SIZE: usize = 256/8;

pub const AEAD_TAG_SIZE: usize = 256/8;

pub const SEED_SIZE: usize = 256/8;
pub const ENCRYPTED_SEED_SIZE: usize = SEED_SIZE + AEAD_TAG_SIZE;

#[derive(Debug)]
pub struct SigningKeyPair(pub Keypair);
#[derive(Debug)]
pub struct SigningPublicKey(pub PublicKey);
#[derive(Debug)]
pub struct SigningSecretKey(pub SecretKey);

impl SigningPublicKey {
    pub fn from_bytes(b: &[u8]) -> Result<Self, SignatureError> {
        Ok(Self(PublicKey::from_bytes(b)?))
    }
    pub fn as_bytes<'a>(&'a self) -> &'a [u8; PUBLIC_KEY_SIZE] {
        self.0.as_bytes()
    }
}

impl SigningSecretKey {
    pub fn from_bytes(entropy: &[u8]) -> Result<Self, Error> {
        ensure!(entropy.len() == SECRET_KEY_SIZE,
                "Incorrect length for ed25519 Signing Secret Key");
        Ok(Self(SecretKey::from_bytes(entropy)?))
    }
}

impl From<&SigningSecretKey> for SigningPublicKey {
    fn from(secret: &SigningSecretKey) -> Self {
        SigningPublicKey(PublicKey::from(&secret.0))
    }
    
}

impl fmt::Display for SigningPublicKey {
     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
         write!(f, "{}",
                hcid::HcidEncoding::with_kind("hcs0")
                .map_err(|_| fmt::Error)?
                .encode(self.as_bytes())
                .map_err(|_| fmt::Error)?
         )
    }
}

impl Serialize for SigningPublicKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("{}", self))
    }
}

impl<'d> Deserialize<'d> for SigningPublicKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'d>,
    {
        let hcid = String::deserialize(deserializer)?;
        Self::from_bytes(
            &hcid::HcidEncoding::with_kind("hcs0")
                .map_err(de::Error::custom)?
                .decode(&hcid)
                .map_err(de::Error::custom)?
        ).map_err(de::Error::custom)
    }
}


// A regular 256-bit seed is encoded in 24 BIP39 words
#[derive(Debug)]
pub struct Seed(pub [u8; SEED_SIZE]);

// An encrypted seed adds a 256-bit authentication MAC + salt, and is encoded in 48 BIP39 words.  In
// holochain_dpki::seed::MnemonicableSeed for EncryptedSeed, we see that a all-0 nonce is used in
// the aead encryption, and the Seed ciphertext is first, followed by the authentication MAC, and
// finally the random salt.5
pub struct EncryptedSeed(pub [u8; ENCRYPTED_SEED_SIZE]);

/*
 * from holochain_dpki::seed
 *
pub(crate) fn pw_enc_zero_nonce(
    data: &[u8],
    passphrase: &mut SecBuf,
    config: Option<PwHashConfig>,
) -> HcResult<EncryptedData> {
    let mut salt = SecBuf::with_insecure(SALTBYTES);
    salt.randomize();
    let mut nonce = SecBuf::with_insecure(NONCEBYTES);
    nonce.write(0, &[0; NONCEBYTES])?;
    let data = pw_enc_base(data, passphrase, &mut salt, &mut nonce, config)?;
    Ok(data)
}

/// Private general wrapper of pw_enc
fn pw_enc_base(
    data: &mut SecBuf,
    passphrase: &mut SecBuf,
    mut salt: &mut SecBuf,
    mut nonce: &mut SecBuf,
    config: Option<PwHashConfig>,
) -> HcResult<EncryptedData> {
    let mut secret = SecBuf::with_secure(kx::SESSIONKEYBYTES);
    let mut cipher = SecBuf::with_insecure(data.len() + aead::ABYTES);
    pw_hash(passphrase, &mut salt, &mut secret, config)?;
    aead::enc(data, &mut secret, None, &mut nonce, &mut cipher)?;

    let salt = salt.read_lock().to_vec();
    let nonce = nonce.read_lock().to_vec();
    let cipher = cipher.read_lock().to_vec();
    // Done
    Ok(EncryptedData {
        salt,
        nonce,
        cipher,
    })
}
 */

pub trait SeedData
where
    Self: Sized,
{
    fn from_bytes(b: &[u8]) -> Result<Self, Error>;
    fn as_bytes<'a>(&'a self) -> &'a [u8];
}

impl SeedData for Seed {
    fn from_bytes(b: &[u8]) -> Result<Self, Error> {
        ensure!(b.len() == SEED_SIZE, "Incorrect length for Seed");
        let mut seed_bytes = [0u8; SEED_SIZE];
        seed_bytes.copy_from_slice(&b[..SEED_SIZE]);        
        Ok(Self(seed_bytes))
    }
    fn as_bytes<'a>(&'a self) -> &'a [u8] {
        &self.0
    }
}

impl Seed {
    pub fn encrypt(
        &self,
        _password: &str, // TODO: implmenent aead encryption!
    ) -> Result<EncryptedSeed, Error> {
        /*
        let encrypted_data =
            pw_enc_zero_nonce(&mut self.seed_mut().buf, &mut passphrase_buf, config)?;
        Ok(EncryptedSeed::new(encrypted_data, self.seed().kind.clone()))
         */
        let mut enc_seed = [0u8; ENCRYPTED_SEED_SIZE];
        enc_seed[..SEED_SIZE].copy_from_slice(self.as_bytes());
        EncryptedSeed::from_bytes(&enc_seed)
    }
}

impl SeedData for EncryptedSeed {
    fn from_bytes(b: &[u8]) -> Result<Self, Error> {
        ensure!(b.len() == ENCRYPTED_SEED_SIZE, "Incorrect length for EncryptedSeed");
        let mut enc_seed_bytes = [0u8; ENCRYPTED_SEED_SIZE];
        enc_seed_bytes.copy_from_slice(&b[..ENCRYPTED_SEED_SIZE]);        
        Ok(Self(enc_seed_bytes))
    }
    fn as_bytes<'a>(&'a self) -> &'a [u8] {
        &self.0
    }
}


/// Create seeds from, or render to Mnemonics, for seeds that are multiples of SEED_SIZE.
pub trait MnemonicableSeed
where
    Self: Sized,
    Self: SeedData,
{
    fn new_from_mnemonic(phrase: String) -> Result<Self, Error> {
        // split out the phrase groups, decode then combine the bytes
        let entropy: Vec<u8> = phrase
            .split(' ')
            .collect::<Vec<&str>>()
            .chunks(MnemonicType::Words24.word_count())
            .map(|chunk| {
                Mnemonic::from_phrase(chunk.join(" "), Language::English)
                    .unwrap()
                    .entropy()
                    .to_owned()
            })
            .flatten()
            .collect();
        // The Vec<u8> contains all of the 24-word chunks of entropy, appended.
        Self::from_bytes(&entropy) // will validate the length.
    }
        
    fn get_mnemonic(&self) -> Result<String, Error> {
        // Split into 256-bit (24-word) chunks
        let mnemonic: String = self.as_bytes()
            .chunks(SEED_SIZE)
            .map(|entropy| {
                Mnemonic::from_entropy(&*entropy, Language::English)
                    .expect("Failed to convert entropy into Mnemonics")
                    .into_phrase()
            })
            .collect::<Vec<String>>()
            .join(" ");
        Ok(mnemonic)
    }
}

impl MnemonicableSeed for Seed { }
impl MnemonicableSeed for EncryptedSeed { }

impl fmt::Display for Seed {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}",
               self.get_mnemonic()
                .map_err(|_e| std::fmt::Error)?)
    }
}

impl fmt::Display for EncryptedSeed {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}",
               self.get_mnemonic()
                .map_err(|_e| std::fmt::Error)?)
    }
}

impl fmt::Debug for EncryptedSeed {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "EncryptedSeed({})", self)
    }
}

impl Serialize for Seed {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("{}", self))
    }
}

impl Serialize for EncryptedSeed {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("{}", self))
    }
}

impl<'d> Deserialize<'d> for Seed {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'d>,
    {
        let mnemonic = String::deserialize(deserializer)?;
        Self::new_from_mnemonic(mnemonic)
            .map_err(de::Error::custom)
    }
}

impl<'d> Deserialize<'d> for EncryptedSeed {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'d>,
    {
        let mnemonic = String::deserialize(deserializer)?;
        Self::new_from_mnemonic(mnemonic)
            .map_err(de::Error::custom)
    }
}


const ARGON2_CONFIG: argon2::Config = argon2::Config {
    variant: argon2::Variant::Argon2id,
    version: argon2::Version::Version13,
    mem_cost: 1 << 16, // 64 MB
    time_cost: 2,
    lanes: 4,
    thread_mode: argon2::ThreadMode::Parallel,
    secret: &[],
    ad: b"holo-config admin ed25519 key v1",
    hash_length: SECRET_KEY_SIZE as u32,
};

/// Convert an email (hashed, as salt) + password + optional u8 array (as pepper, to uniquify the
/// result, in situations where the same email+password might be used) into a Seed.  The SHA-512 of
/// the email allows us to use email addresses shorter than 8 bytes.
pub fn email_password_to_seed(
    email: &str,
    password: &str,
    pepper: Option<&[u8]>,
) -> Result<Seed, Error> {
    let salt = Sha512::digest(email.as_bytes());

    let mut config = ARGON2_CONFIG.clone();
    if let Some(secret) = pepper {
        config.secret = secret;
    }

    Seed::from_bytes(
        &argon2::hash_raw(
            &password.as_bytes(), &salt, &config
        )?
    )
}

/// Use some seed entropy to generate an ed25519 Signing keypair.  Note that this retains the raw
/// entropy as the "SecretKey", and generates the PublicKey by A) producing a 256-bit hash from the
/// entropy, splitting it into a 256-bit Scalar, and adjusting it to be valid (<2^255, ie. top bit
/// clear, etc.).  Therefore, the Public key will be consistent with libsodium.  *However*, the
/// Secret key will *not*, as libsodium adjusts it to be a valid Scalar before returning it!  Other
/// libraries (like ed25519_dalek) adjust the secret key each time, before signing with it.
pub fn signing_keypair_from_seed(
    entropy: &[u8]
) -> Result<(SigningPublicKey, SigningSecretKey), Error> {
    let secret = SigningSecretKey::from_bytes(entropy)?;
    Ok(((&secret).into(), secret))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::ExpandedSecretKey;
    use std::convert::From;

    #[test]
    fn keypair_should_generate_consistent_keys() {
        let (public, secret) = signing_keypair_from_seed(&[
             0u8,  1u8,  2u8,  3u8,  4u8,  5u8,  6u8,  7u8,  8u8,  9u8,
            10u8, 11u8, 12u8, 13u8, 14u8, 15u8, 16u8, 17u8, 18u8, 19u8,
            20u8, 21u8, 22u8, 23u8, 24u8, 25u8, 26u8, 27u8, 28u8, 29u8,
            30u8,255u8,
        ]).expect("Failed to generate keypair");
        assert_eq!(public.to_string(), "HcSciPgAEa7N4e6os7X7zK4JdbXnmxygkVVkHChDT3cbuh3wByfwzx9SNuo9xbz");
        assert_eq!(format!("{:?}", secret),
                   "SigningSecretKey(SecretKey: [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 255])");
        // However, what holochain-rust de/serializes is the *expanded* secret key, after SHA-512,
        // curve-fitting and nonce-generation, as 64 bytes (Scalar + Nonce).  This theory isn't
        // correct, as the expanded secret key differs from what is output by libsodium on
        // holochain-rust in dpki/src/keypair.rs:
        assert_eq!(format!("{:?}", &ExpandedSecretKey::from(&secret.0).to_bytes()[..]),
                   "[232, 104, 184, 229, 117, 139, 31, 226, 138, 101, 254, 105, 221, 111, 217, 14, 215, 71, 80, 59, 22, 147, 1, 202, 32, 158, 33, 165, 240, 111, 237, 104, 16, 113, 215, 44, 243, 69, 155, 40, 97, 222, 188, 61, 170, 121, 22, 210, 97, 161, 3, 47, 16, 168, 249, 196, 121, 101, 26, 59, 108, 221, 212, 100]"        );
        
    }
}
