
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
    Sha256, Sha512, Digest
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
    pub fn from_bytes(b: &[u8]) -> Result<Self, SignatureError> {
        Ok(Self(SecretKey::from_bytes(b)?))
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
    let salt = Sha256::digest(email.as_bytes());

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

/// Use some seed entropy to generate an ed25519 Signing keypair.  To maintain comptibility with
/// libsodium-based ed25519, we will use the same SHA-512 hash of the incoming 256-bit seed entropy,
/// and then consume the first 256 bits of it to generate the ed25519 secret key.
pub fn signing_keypair_from_seed(
    entropy: &[u8] // Typically should be 32 bytes of entropy, for libsodium compatibility
) -> Result<(SigningPublicKey, SigningSecretKey), Error> {
    let digest = Sha512::digest(entropy);
    let secret = SigningSecretKey::from_bytes(&digest[..SECRET_KEY_SIZE])?;
    let public = SigningPublicKey::from(&secret);
    Ok((public, secret))
}
    
