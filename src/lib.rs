#[macro_use] extern crate failure;

pub mod config;
pub mod dpki;

pub use config::{
    ConfigResult, Config, ConfigSeed,
    AdminSigningPublicKey,
};
pub use dpki::{
    Seed, SeedData, MnemonicableSeed,
    SigningPublicKey,    
};
