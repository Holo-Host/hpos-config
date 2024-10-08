#[cfg(test)]
mod tests {

    use ed25519_dalek::VerifyingKey;
    use hpos_config_core::Config;

    #[tokio::test(flavor = "multi_thread")]
    async fn test_hpos_config() -> Result<(), String> {
        // emulate the UI

        let master = hc_seed_bundle::UnlockedSeedBundle::new_random()
            .await
            .unwrap();

        let passphrase = sodoken::BufRead::from(b"test-passphrase".to_vec());
        let revocation_bundle = master.derive(0).await.unwrap();
        let revocation_pub_key = revocation_bundle.get_sign_pub_key().read_lock().to_vec();

        let device_derivation_path = 2;
        let device_bundle = master.derive(device_derivation_path).await.unwrap();
        let device_bundle_encoded_bytes = device_bundle
            .lock()
            .add_pwhash_cipher(passphrase)
            .lock()
            .await
            .unwrap();
        let device_bundle_base64 = base64::encode(&device_bundle_encoded_bytes);

        // derive the holoport ID

        let holoport_id = device_bundle.derive(1).await.unwrap();

        let holoport_id = holoport_id.get_sign_pub_key().read_lock().to_vec();

        // initialize a new Config struct
        let email = "joel@holo.host".to_string();
        let password = "password".to_string();
        let registration_code = "registration-code".to_string();
        let rev_key_bytes = revocation_pub_key[0..32].try_into().unwrap();
        let revocation_pub_key = VerifyingKey::from_bytes(&rev_key_bytes).unwrap();
        let holoport_id_bytes = holoport_id[0..32].try_into().unwrap();
        let holoport_id = VerifyingKey::from_bytes(&holoport_id_bytes).unwrap();
        let hpos_config = Config::new(
            email.clone(),
            password,
            registration_code,
            revocation_pub_key,
            device_derivation_path.to_string(),
            device_bundle_base64.clone(),
            holoport_id,
        )
        .unwrap();

        assert_eq!(hpos_config.1, holoport_id.clone());

        println!("{}", serde_json::to_string_pretty(&hpos_config.0).unwrap());

        if let Config::V3 {
            device_bundle,
            device_derivation_path,
            revocation_pub_key,
            holoport_id,
            initial_host_pub_key,
            registration_code,
            settings,
        } = hpos_config.0
        {
            assert_eq!(device_bundle, device_bundle_base64,);
            assert_eq!(device_derivation_path, device_derivation_path.to_string());
            assert_eq!(revocation_pub_key, revocation_pub_key);
            assert_eq!(holoport_id, holoport_id);
            assert_eq!(registration_code, registration_code);
            assert_eq!(settings.admin.email, email);
            return Ok(());
        } else {
            return Err("Expected V3 variant".to_string());
        }
    }
}
