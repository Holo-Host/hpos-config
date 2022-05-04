# hpos-config

This repo contains
- A web UI called Quickstart
- A Rust library called `hpos-config-core`
- A bunch of utility binaries for interacting with the data structures provided by `hpos-config-core`
    - (`gen-cli`, `into-base36-id`, `is-valid`, `seed-bundle-explorer`, `seed-encoder`)

Quickstart UI allows registered HoloPort owners to generate configuration files and private keys for their HoloPorts and walks them through the set up process.

`hpos-config-core` contains the structure of initial HoloPort configuration files and the keypair generation algorithms used during Quickstart. It is used as a dependency by Quickstart UI through Wasm, as well as by various other services that run on the HoloPort. For a (hopefully) complete list of downstream dependents, see [hpos-config-core](./core/README.md).

The production copy of Quickstart UI is deployed at https://quickstart.holo.host/. The staging/development copy is deployed at https://holo-host.github.io/hpos-config and follows the `gh-pages` branch of this repo.


## Quickstart UI Development

Quickstart UI is written in vanilla JS and does not use any web framework. The code lives in the [gen-web](./gen-web) folder of this repository.

### Membrane Proof Service

After the user enters a Registration Code, Quickstart makes a request to a Membrane Proof Service (see [this function](https://github.com/Holo-Host/hpos-config/blob/8c25e644dd60b544af4dc2a9e93144aabdc5df97/gen-web/src/index.js#L515)). When working with Quickstart, you need to choose which Membrane Proof Service to use.

- You can spin up a local copy of the [`membrane-proof-service`](https://github.com/Holo-Host/holo-nixpkgs/tree/develop/overlays/holo-nixpkgs/membrane-proof-service), and use it through `http://localhost:8800`
    - [Setup instructions for membrane-proof-service](https://github.com/Holo-Host/holo-nixpkgs/blob/e9f7eea48954a7937b36d58a41616457557b3b59/overlays/holo-nixpkgs/membrane-proof-service/README.md#development)
    - [See this file for email and registration code](https://github.com/Holo-Host/holo-nixpkgs/blob/develop/overlays/holo-nixpkgs/membrane-proof-service/tests/test-preload-db.js), which you'll need when walking through the UI.
- You can use the development instance of `membrane-proof-service` (currently deployed at <https://devnet-membrane-proof-service.holo.host/>).

### Serving the UI

(inside `gen-web`)

Once you've picked a Membrane Proof Service (e.g `http://localhost:8800`), you can run the following command to serve the UI in **optimized** mode. (Takes longer to build each time)

```
MEMBRANE_PROOF_SERVICE_URL=http://localhost:8800 yarn serve
```

You can serve the UI in **unoptimized** mode using
```
yarn start
```

Note: [`yarn start` currently has `MEMBRANE_PROOF_SERVICE_URL=http://localhost:8800` hardcoded.](https://github.com/Holo-Host/hpos-config/blob/8c25e644dd60b544af4dc2a9e93144aabdc5df97/gen-web/package.json#L6)

If you encounter the "Generating your Keys" page taking a long time in unoptimized mode, try switching to optimized. There seems to be some issue with the unoptimized Wasm.

### Deploying the UI to GitHub Pages

You need to push a commit to the `gh-pages` branch that contains the built files from `gen-web/dist/`. Here is an example of the exact commands you could use to deploy. (Perhaps this should be made into a script)

```
COMMIT_HASH="$(git rev-parse HEAD)"
cd gen-web
yarn install
rm -r dist/
MEMBRANE_PROOF_SERVICE_URL=https://test-membrane-proof-service.holo.host/ yarn build
git checkout gh-pages
cd ..
git rm assets/ *.js *.wasm *.css *.png *.html
mv gen-web/dist/* .
git commit -a -m "Deploy $COMMIT_HASH (https://test-membrane-proof-service.holo.host/)"
git push
```
