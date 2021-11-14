/**
 * Validate if string is valid size
 * @param {string} registrationCode
 */
export const validateRegistrationCode = (registrationCode) => {
// TODO: define what the min size of the registration code will be
// TODO: check if the size requirement is met
return registrationCode !== ""
}

/**
 * Validate if string is valid size
 * @param {string} registrationCode
 */
 export const validatePassphrae = (passphrase) => {
    return passphrase.length > 20
}
    
/**	
 * Validate device by size of screen	
 */	
export const validateScreenSize = () => {	
/* Detect whether on laptop or desktop */	
return (window.screen.availWidth >= 768)
}

/**	
 * Validate device by confirming non-mobile user agent	
 */	
export const detectMobileUserAgent = () => {	
/* Detect whether on mobile browser */	
return (/(android|bb\d+|meego).+mobile|avantgo|bada\/|blackberry|blazer|compal|elaine|fennec|hiptop|iemobile|ip(hone|od)|iris|kindle|lge |maemo|midp|mmp|mobile.+firefox|netfront|opera m(ob|in)i|palm( os)?|phone|p(ixi|re)\/|plucker|pocket|psp|series(4|6)0|symbian|treo|up\.(browser|link)|vodafone|wap|windows ce|xda|xiino/i.test(navigator.userAgent))
}

/**
 * Validate if string is email (super simple because actual validation is via sent email)
 * @param {string} email
 */
export const validateEmail = (email) => {
const re = /[^@]+@[^\.]+\..+/g
return re.test(String(email).toLowerCase())
}