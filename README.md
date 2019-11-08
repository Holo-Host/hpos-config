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

## Building & Generating a `hpos-state.json`

We will generate a `Config` object in JSON form, to be saved into `hpos-state.json`:

```
$ nix-build -A hpos-state-gen-cli
$ ./target/debug/hpos-state-gen-cli  --email "a@b.ca" --password "secret" | tee hpos-state.json
```

Also available is the nix-shell and manual build approach:
```
$ nix-shell
$ cargo build --release --bin hpos-state-gen-cli

$ ./target/release/hpos-state-gen-cli --email "a@b.ca" --password "secret" | tee hpos-state.json
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

## Building a Web UI to Generate Config

Each UI can build and ship exactly the subset of the Rust `hpos-state` project required to support
its functionality.  We do not ship a "standard" JS library, but instead allow the Web UI developer
to write a very small Rust API calling hpos-state code, which is compiled to a small WASM static
asset included with and called by the Web UI project.

For example, the provided `gen-web` example generates a JSON string containing a hpos-state,
from a supplied email and password.  The Rust code:

```
// https://github.com/rustwasm/wasm-bindgen/issues/1004
fn config_raw(email: String, password: String) -> Result<JsValue, Error> {
    let (config, public_key) = Config::new(email, password, None)?;

    let config_data = ConfigData {
        config: serde_json::to_string_pretty(&config)?,
        url: public_key::to_url(&public_key)?.into_string()
    };

    Ok(JsValue::from_serde(&config_data)?)
}
```

is compiled using the Javascript `@wasm-tool/wasm-pack-plugin`.  A very simple Javascript `index.js`
loads the compiled WASM package:

```
import { saveAs } from 'file-saver';
async function main() {
  const { config } = await import('./pkg');
  const elements = {
    generate: document.querySelector('#generate'),
    ...
  }
  ...
  elements.generate.addEventListener('click', e => {
    const config_data = config(elements.email.value, elements.password.value);
    const blob = new Blob([config_data.config], {type: 'application/json'});

    saveAs(blob, 'hpos-state.json');
    alert(config_data.url);
  });
};
main();
```

When the Webpack-compiled page is loaded, the DOM is configured by the above Javascript, and the
WASM code is invoked on the Generate button-click, producing the `hpos-state.json` file.

### Building the WASM and JS

To build an example web UI, able to call a WASM-compiled function that can generate and return a
`Config` in JSON form suitable for saving to `hpos-state.json`:

```
$ nix-shell
$ cd gen-web
$ npm install
$ npm build
$ npm run serve
```

Go to `http://localhost:8080`, type in an email and password, and click `Generate`, and save the
file.  Will default to saving a file named `hpos-state.json` to your downloads directory.

## Generating a Holochain Keystore from `hpos-state.json`

To use the seed saved in `hpos-state.json` from within a Holochain application (for example, upon
start-up of the Holochain Conductor on the HoloPort), the Config needs to be deserialized, and the
seed used in the standard Holochain cryptography routines.

Standard Rust Serialize/Deserialize functionality is provided:

```
use hpos_state_core::{config::Seed, Config}
...
let Config::V1 { seed, .. } = serde_json::from_reader(stdin())?;
```

Generate a `hpos-state.json`, and use `hpos-state-derive-keystore` to load it and generate a Holochain
keystore:

```
$ nix-shell
$ cargo build --release --bin hpos-state-derive-keystore < hpos-state.json
$ ./target/release/hpos-state-derive-keystore < hpos-state.json
HcSCjwu4wIi4BawccpoEINNfsybv76wrqoJe39y4KNAO83gsd87mKIU7Tfjy7ci
{
  "passphrase_check": "eyJzY...0=",
  "secrets": {
    "primary_keybundle:enc_key": {
      "blob_type": "EncryptingKey",
      "seed_type": "Mock",
      "hint": "",
      "data": "eyJzY...0="
    },
    "primary_keybundle:sign_key": {
      "blob_type": "SigningKey",
      "seed_type": "Mock",
      "hint": "",
      "data": "eyJzYW...19"
    },
    "root_seed": {
      "blob_type": "Seed",
      "seed_type": "OneShot",
      "hint": "",
      "data": "eyJzYW...19"
    }
  }
}
```
