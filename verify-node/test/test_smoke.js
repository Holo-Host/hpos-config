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

// Get an admin Keypair
//keypair = 
//verify.verify( 
