use ed25519_dalek::VerifyingKey;
use failure::*;
use url::Url;

pub fn to_base36_id(public_key: &VerifyingKey) -> String {
    base36::encode(&public_key.to_bytes())
}

pub fn to_url(public_key: &VerifyingKey) -> Fallible<Url> {
    let url = format!("https://{}.holohost.net", to_base36_id(public_key));
    Ok(Url::parse(&url)?)
}

/// internal compute a 16 byte blake2b hash
fn blake2b_128(data: &[u8]) -> Vec<u8> {
    let hash = blake2b_simd::Params::new().hash_length(16).hash(data);
    hash.as_bytes().to_vec()
}

pub fn holo_dht_location_bytes(data: &[u8]) -> Vec<u8> {
    // Assert the data size is relatively small so we are
    // comfortable executing this synchronously / blocking tokio thread.
    assert_eq!(32, data.len(), "only 32 byte hashes supported");

    let hash = blake2b_128(data);
    let mut out = vec![hash[0], hash[1], hash[2], hash[3]];
    for i in (4..16).step_by(4) {
        out[0] ^= hash[i];
        out[1] ^= hash[i + 1];
        out[2] ^= hash[i + 2];
        out[3] ^= hash[i + 3];
    }
    out
}

pub(crate) const AGENT_PREFIX: &[u8] = &[0x84, 0x20, 0x24]; // uhCAk [132, 32, 36]

/// convert public key to holochain compatible format
pub fn to_holochain_encoded_agent_key(public_key: &VerifyingKey) -> String {
    let x: [u8; 32] = public_key.to_bytes();
    format!(
        "u{}",
        base64::encode_config(
            [AGENT_PREFIX, &x, &holo_dht_location_bytes(x.as_ref())].concat(),
            base64::URL_SAFE_NO_PAD
        )
    )
}
