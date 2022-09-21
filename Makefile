deploy: 
	cd gen-web; npm install; MEMBRANE_PROOF_SERVICE_URL=https://hbs.holo.host npm run build; wrangler publish --env production

deploy-dev:
	cd gen-web; npm install; MEMBRANE_PROOF_SERVICE_URL=https://hbs.dev.holotest.net npm run build; wrangler publish