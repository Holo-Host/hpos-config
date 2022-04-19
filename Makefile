deploy: 
	cd gen-web; npm install; MEMBRANE_PROOF_SERVICE_URL=https://membrane-proof-service.holo.host/ npm run build; wrangler publish --env production

deploy-dev:
	cd gen-web; npm install; MEMBRANE_PROOF_SERVICE_URL=https://test-membrane-proof-service.holo.host/ npm run build; wrangler publish