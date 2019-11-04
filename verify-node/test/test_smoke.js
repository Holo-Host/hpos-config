const verify = require('../pkg/holo_config_verify_node.js');

// Call wasm method 'add' typical stack method
let result = verify.add(10, 2);
console.log('add result:' + result);
// Now let's access heap memory /reference
var a = new Uint8Array(100);
a[0] = 225;
a[1] = 10;
console.log('pre remote call a[1] === ' + a[1]);
verify.alter(a);
console.log('post remote call a[1] === ' + a[1]);

// Get an admin PublicKey, and a message (as Uint8Array) and Signature, and test them
var enc = new TextEncoder();
var verified = verify.verify(
    "pSh6Jyz3bJk1bRlrutG3+2kSJb6BKn1m0OHJ2AL3E+8",
    enc.encode("Of the increase of his government and peace there shall be no end"),
    "z23Gmz6w6C4gNdx3gm1Ta02xns11NtASNPNiFAe9QHdo7TK4NIfTWW0qvjpH2o7b1/DxzeBi2+20xF/Un8i8BQ"
)
console.log(`Signature Verification: ${verified}`)
