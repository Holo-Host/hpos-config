use ed25519_dalek::PublicKey;
use failure::*;
use url::Url;

pub fn to_base36_id(public_key: &PublicKey) -> String {
    base36::encode(&public_key.to_bytes())
}

pub fn to_url(public_key: &PublicKey) -> Fallible<Url> {
    let url = format!("https://{}.holohost.net", to_base36_id(&public_key));
    Ok(Url::parse(&url)?)
}
