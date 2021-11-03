/**
 * Util functions 
 * 
 * /

/**
 *  converts encoded bytes to base64 on browser
 * @param {array} encodedBytes
 */

import { Base64 } from 'js-base64';
// import _sodium from 'libsodium-wrappers'

export const toBase64 = (encodedBytes) => {
    let u8a = new Uint8Array(encodedBytes);
    return Base64.fromUint8Array(u8a, true);
//   return _sodium.to_base64(encodedBytes, _sodium.base64_variants.URLSAFE_NO_PADDING)
}


const FILE_PREFIX = "hpos-config"
const FILE_TYPE = ".json"

/**	
 * generate file name based on the device number
 * @param {number} deviceNumber
 * @param {string} pubKey	
 */	
export const genConfigFileName = (deviceNumber, pubKey) => {
    if (deviceNumber == 0) {
        return `${FILE_PREFIX}-primary${FILE_TYPE}`
    } else {
        return `${FILE_PREFIX}-secondary-${pubKey.substring(0, 5)}${FILE_TYPE}`    
    }
}