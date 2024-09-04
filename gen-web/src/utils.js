/**
 * Util functions 
 * 
 * /

/**
 *  converts encoded bytes to base64 on browser
 * @param {array} encodedBytes
 */

import _sodium from 'libsodium-wrappers'

export const toBase64 = (encodedBytes) => {
    return _sodium.to_base64(encodedBytes, _sodium.base64_variants.URLSAFE_NO_PADDING)
}


const FILE_PREFIX = "hp-config"
const FILE_TYPE = ".json"

/**	
 * generate file name based on the device number
 * @param {number} deviceNumber
 * @param {string} pubKey	
 */
export const genConfigFileName = (pubKey) => {
    return `${FILE_PREFIX}-${pubKey.substring(0, 5)}${FILE_TYPE}`

}