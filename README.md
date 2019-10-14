# Configure a Holo HoloPortOS Instance

Deploying a basic configuration to a HoloPortOS instance requires key generation material and basic
identity + password to be collected and deployed.

## USB Configuration

The simplest and most direct method is to generate a configuration, and copy it onto a USB stick,
which is then inserted into the HoloPortOS instance.  When the device boots, it will:

- Use the data on the USB stick to create its Holochain and potentially other keys
- Authenticate itself to the Holo ZeroTier network, which will provision its DNS configuration
- Start the Holo services
- Eject the USB and blacklist the kernel modules

## Building & Generating a `holo-config.json`

We'll generate a `Config` object in JSON form, to be saved into `holo-config.json`:

```
$ nix-build -A holo-config-generate-cli
$ ./target/debug/holo-config-generate-cli  --email "a@b.ca" --password "secret" | tee holo-config.json
```

Also available is the nix-shell and manual build approach:
```
$ nix-shell
$ cargo build --release --bin holo-config-generate-cli

$ ./target/release/holo-config-generate-cli --email "a@b.ca" --password "secret" | tee holo-config.json
https://hcscjzpwmnr6ezxybxauytg458vgr6t8nuj3deyd3g6exybqydgsz38qc8n3zfr.holohost.net/
{
  "v1": {
  "seed": "jYvZ70UkYJGjMzADb4PcQzHcECLfUHHXb9KMk6NY2fE",
  "admin": {
    "email": "a@b.ca",
    "public_key": "4sfPilERj9dPCkTADmJ8MfsUkfXOxWOlPHhhtVuzlt4"
  }
}
```

