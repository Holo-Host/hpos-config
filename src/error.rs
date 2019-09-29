use crypto::symmetriccipher::SymmetricCipherError;
use ed25519_dalek::SignatureError;
use hcid::HcidError;
use std::convert::From;
use std::time::SystemTimeError;
#[derive(Debug)]
pub enum ConfigurationError {
    Generic(String),
    SignatureError(SignatureError),
    HcidError(HcidError),
    SystemTimeError(SystemTimeError),
    Argon2Error(argon2::Error),
    SymmetricCipherError(SymmetricCipherError),
    Base64DecodeError(base64::DecodeError),
}

impl From<HcidError> for ConfigurationError {
    fn from(e: HcidError) -> Self {
        ConfigurationError::HcidError(e)
    }
}

impl From<SystemTimeError> for ConfigurationError {
    fn from(e: SystemTimeError) -> Self {
        ConfigurationError::SystemTimeError(e)
    }
}

impl From<SignatureError> for ConfigurationError {
    fn from(e: SignatureError) -> Self {
        ConfigurationError::SignatureError(e)
    }
}

impl From<argon2::Error> for ConfigurationError {
    fn from(e: argon2::Error) -> Self {
        ConfigurationError::Argon2Error(e)
    }
}

impl From<SymmetricCipherError> for ConfigurationError {
    fn from(e: SymmetricCipherError) -> Self {
        ConfigurationError::SymmetricCipherError(e)
    }
}

impl From<base64::DecodeError> for ConfigurationError {
    fn from(e: base64::DecodeError) -> Self {
        ConfigurationError::Base64DecodeError(e)
    }
}

impl ConfigurationError {
    pub fn new(msg: &str) -> Self {
        ConfigurationError::Generic(msg.to_string())
    }
}

impl std::error::Error for ConfigurationError {
    fn description(&self) -> &str {
        "ConfigurationError"
    }
}

impl std::fmt::Display for ConfigurationError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// represents a Result object returned by an api in the cryptography system
pub type ConfigurationResult<T> = Result<T, ConfigurationError>;
