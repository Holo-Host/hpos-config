const verify_node = require('../pkg/holo_config_verify_node.js');


// Get an admin PublicKey, and a message (as Uint8Array) and Signature, and test them
var enc = new TextEncoder();
var admin_pubkey = "HcACjJJIqiut375nue4x4gMMYmI5q85jcit57aJkQwUobzpj5abqPe9QsEiivgz";
var message = "Of the increase of his government and peace there shall be no end";
var message_sig = "z23Gmz6w6C4gNdx3gm1Ta02xns11NtASNPNiFAe9QHdo7TK4NIfTWW0qvjpH2o7b1/DxzeBi2+20xF/Un8i8BQ"

// Use an AdminVerifier to encapsulate an AdminPublicKey and verify with it
var verifier = new verify_node.AdminVerifier( admin_pubkey );
var verifier_verified = verifier.verify( enc.encode(message), message_sig );
console.log(`Admin Signature Verifier ok:  ${verifier_verified}`)

var verifier_rejected = verifier.verify( enc.encode(message + "X"), message_sig );
console.log(`Admin Signature Verifier bad: ${verifier_rejected}`)
