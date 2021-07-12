use ed25519_dalek::PublicKey;
use failure::*;
use url::Url;

// ? Should we encode it according to the proposed plan here:
// ? https://github.com/holochain/hc-utils/pull/15
pub fn to_url(public_key: &PublicKey) -> Fallible<Url> {
    let url = format!(
        "https://{}.holohost.net",
        holochain_pub_key_encoding(&public_key.to_bytes())
    );

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

pub(crate) const AGENT_PREFIX: &str = "hcak";

// The same encoding that is used in hc-utils
pub fn holochain_pub_key_encoding(x: &[u8]) -> String {
    format!(
        "{}{}",
        AGENT_PREFIX,
        multibase::encode(
            multibase::Base::Base32Lower,
            &[x, &holo_dht_location_bytes(x.as_ref())].concat(),
        )
    )
}
