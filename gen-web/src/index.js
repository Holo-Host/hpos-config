
(async () => {
  const filesaver = require('file-saver');
  const { config } = await import('../pkg')
  const hcSeedBundle = await import('@holochain/hc-seed-bundle')
  const DOWNLOAD_FILE_NAME = 'hpos-config.json'

  let stepTracker
  let signalKeyGen = false
  let resetUserConfig = false
  let downloadTracker = {}
  let configFileBlob = ''

  /* Parse HTML elements */
  const buttons = {
    startPrep: document.querySelector('#start-prep-button'),
    start: document.querySelector('#start-button'),
    generate: document.querySelector('#generate-button'),
    plugInDrive: document.querySelector('#drive-plugin-button'),
    termsAndConditions: document.querySelector('#terms-and-conditions'),
    download: document.querySelector('#download-button'),
    postDownload: document.querySelector('#post-download-button'),
    copied: document.querySelector('#copied-button'),
    closeNotice: document.querySelector('#close-notice'),
    back0b: document.querySelector('#back-button0b'),
    back1: document.querySelector('#back-button1'),
    back2: document.querySelector('#back-button2'),
    back2Confirmation: document.querySelector('#back-button2-confirmation'),
    back3: document.querySelector('#back-button3'),
    back4: document.querySelector('#back-button4'),
    back5: document.querySelector('#back-button5'),
    forumHelp: document.querySelector('#forum-help')
  }

  const inputs = {
    registrationCode: document.querySelector('#registration-code'),
    email: document.querySelector('#email'),
    password: document.querySelector('#password'),
    passwordCheck: document.querySelector('#password-check'),
  }

  const inlineVariables = {
    contentContainer: document.querySelector('#content-container'),
    registrationCodeInputArea: document.querySelector('#registration-code-form-item'),
    emailPlaceholder: document.querySelector('#email-placeholder'),
    emailInputArea: document.querySelector('#email-form-item'),
    passwordInputArea: document.querySelector('#password-form-item'),
    passwordCheckInputArea: document.querySelector('#password-check-form-item'),
    formErrorMessage: document.querySelector('#form-error-message'),
    downloadFileName: document.querySelector('#download-file')
  }

  const errorMessages = {
    missingFields: 'Please complete missing fields.',
    registrationCode: 'Invalid code',
    email: 'Email domain not recognized',
    password: 'Your password needs to be at least eight character in length',
    passwordCheck: 'Passwords do not match',
    generateConfig: 'An error occured when configuring your user file. Please update your information and try again.'
  }

  const user = {
    registrationCode: '',
    email: '',
    password: ''
  }


  /** Actions executed at button click
  * ======================================
  */
  const click = {
    startPrep: () => {
      if (!validateScreenSize() || detectMobileUserAgent()) {
        const confirmed = confirm('This experience has not been optimized for mobile devices. Please continue only if you are using a laptop or PC.\n\nContinuing on a mobile device may result in unexpected issues.')
        if (confirmed === true) return updateUiStep(0.5)
        else return null
      } else {
        // TODO: RESET TO BELOW ONCE OUT OF DEV MODE
        updateUiStep(0.5)
  
        // DEV MODE HACK TO SWITCH THROUGH PAGES
        // updateUiStep(4)
      }
    },
    start: () => {
      updateUiStep(1)
      inputs.email.click()
    },  
    generate: async () => {
      signalKeyGen = true
      const inputValidity = await verifyInputData()
      if (!inputValidity) return buttons.generate.disabled = true
      
      /* Set user config */
      user.registrationCode = inputs.registrationCode.value
      user.email = inputs.email.value
      user.password = inputs.password.value

      // DEV MODE - Config Check: 
      // console.log('user config : ', user)
      
      /* Communicate visually that something is happening in the bkgd */
      buttons.generate.disabled = true
      downloadTracker = false
      click.openLoader()
      // // get passphrase
      // await hcSeedBundle.seedBundleReady
      // // generate a new pure entropy master seed
      // const master = hcSeedBundle.UnlockedSeedBundle.newRandom({
      //   bundleType: 'master'
      // })

      // // we need the passphrase as a Uint8Array
      // const pw = (new TextEncoder()).encode('test-passphrase')
      // const encodedBytes = master.lock([
      //   new hcSeedBundle.SeedCipherPwHash(
      //     hcSeedBundle.parseSecret(pw), 'interactive')
      // ])
      // console.log(">>>>>>>>>>", encodedBytes);
      // // clear our secrets
      // master.zero()
      // // call hc-seed-bundle 
      setTimeout(() => {
        // Generate hpos-config.json and create download blob attached to url
        try {
          inlineVariables.formErrorMessage.innerHTML = ''
          generateBlob(user, buttons.download)
        } catch (e) {
          inlineVariables.formErrorMessage.innerHTML = errorMessages.generateConfig
          throw new Error(`Error executing generateBlob with an error.  Error: ${e}`)
        }

        /* Clean State */
        buttons.generate.disabled = false
        click.closeLoader()
        updateUiStep(2)
        updateProgressBar(1)

        /* Reset Password inputs */
        inputs.password.value = ''
        inputs.passwordCheck.value = ''
      }, 1500)
    },
    plugInDrive: () => {
      updateUiStep(3)
      updateProgressBar(2)
    },
    termsAndConditions: e => {
      e.preventDefault()
      window.open(
        'https://holo.host/alpha-terms',
        '_blank'
      )
    },
    download: async () => {      
      /* Communicate visually that something is happening in the bkgd */
        buttons.download.classList.add('disabled')
        buttons.download.disabled = true
        buttons.download.innerHTML = 'Saving Configuration File...'

        setTimeout(() => {
          try {
            filesaver.saveAs(configFileBlob, DOWNLOAD_FILE_NAME)
          } catch (e) {
            throw new Error(`Error saving config. Error: ${e}`)
          }

          /* Clean State */
          downloadTracker = true
          buttons.download.classList.remove('disabled')
          buttons.download.disabled = false
          buttons.download.innerHTML = 'Save Configuration File Again'
          verifyDownloadComplete(downloadTracker)
        }, 1000)
    },
    postDownload: () => {  
      updateUiStep(4)
      updateProgressBar(3)
    },
    copied: () => {
      updateUiStep(5)
      updateProgressBar(4)
    },
    openLoader: () => {
      document.querySelector('#fixed-overlay-loader').style.display = 'block'
      document.querySelector('#modal-overlay-loader').style.display = 'block'
    },
    closeLoader: () => {
      document.querySelector('#fixed-overlay-loader').style.display = 'none'
      document.querySelector('#modal-overlay-loader').style.display = 'none'
    },
    openNotice: () => {
      document.querySelector('#fixed-overlay-notice').style.display = 'block'
      document.querySelector('#modal-overlay-notice').style.display = 'block'
    },
    closeNotice: () => {
      document.querySelector('#fixed-overlay-notice').style.display = 'none'
      document.querySelector('#modal-overlay-notice').style.display = 'none'
    },
    back0b: () => {
      updateUiStep(0)
    },
    back1: () => {
      updateUiStep(0.5)
    },
    back2: () => {
      click.openNotice()
    },
    back2Confirmation: () => {
      click.closeNotice()
      resetUserConfig = true
      const rewind = true
      updateProgressBar(2, rewind)
      updateUiStep(1)
    },
    back3: () => {
      const rewind = true
      updateProgressBar(3, rewind)
      updateUiStep(2)
    },
    back4: () => {
      const rewind = true
      updateProgressBar(4, rewind)
      updateUiStep(3)
    },
    back5: () => {
      const rewind = true
      updateProgressBar(5, rewind)
      updateUiStep(4)
    },
    forumHelp: e => {
      e.preventDefault()
      window.open(
        'https://forum.holo.host',
        '_blank'
      )
    },
    handleEnter: event => {
      const step = stepTracker || 0
      const keycode = (event.keyCode ? event.keyCode : event.which)
      /* Number 13 is the "Enter" key on the keyboard */
      if (keycode === 13 && step <= 4) {
        const stepButton = buttonBystep[step]
        stepButton.click()
      }
      else return null
    },
    activateInput: event => {
      let labelId
      if (event.target.id.includes('label')) labelId = document.querySelector(`#${event.target.id}`)
      else {
        const inputId = event.target.id 
        labelId = document.querySelector(`#${inputId}-label`)
      }
      
      const activeInputs = document.querySelectorAll('.input-active')
      if (activeInputs) {
        for (let activeInput of activeInputs) {
          if (!activeInput.parentElement.querySelector('input').value){
            activeInput.classList.remove('input-active')
            activeInput.dataset.shrink = 'false'
          }
        }
      }

      if (labelId) {
        labelId.classList.add('input-active')
        labelId.dataset.shrink = 'true'
      }
      
      verifyInputData()
	},
	  confirmValidInput: () => confirmValidInput()
  }

  /* Bind keystroke action to listener */
  document.querySelector('body').onkeyup = click.handleEnter

  /* Set intial 'disable' state for all config actions buttons */
  buttons.generate.disabled = false
  buttons.postDownload.disabled = true

  /* Bind actions to buttons */
  buttons.startPrep.onclick = click.startPrep
  buttons.start.onclick = click.start
  buttons.generate.onclick = click.generate
  buttons.termsAndConditions.onclick = click.termsAndConditions
  buttons.download.onclick = click.download
  buttons.postDownload.onclick = click.postDownload
  buttons.copied.onclick = click.copied
  buttons.plugInDrive.onclick = click.plugInDrive
  buttons.closeNotice.onclick = click.closeNotice
  buttons.back0b.onclick = click.back0b
  buttons.back1.onclick = click.back1
  buttons.back2.onclick = click.back2
  buttons.back2Confirmation.onclick = click.back2Confirmation
  buttons.back3.onclick = click.back3
  buttons.back4.onclick = click.back4
  buttons.back5.onclick = click.back5
  buttons.forumHelp.onclick = click.forumHelp
  document.onkeyup = click.activateInput
  /* Bind input actions to inputArea actions */
  inlineVariables.registrationCodeInputArea.onclick = e => { inputs.registrationCode.focus(); return click.activateInput(e) }
  inlineVariables.emailInputArea.onclick = e => { inputs.email.focus(); return click.activateInput(e) }
  inlineVariables.passwordInputArea.onclick = e => { inputs.password.focus(); return click.activateInput(e) }
  inlineVariables.passwordCheckInputArea.onclick = e => { inputs.passwordCheck.focus(); return click.activateInput(e) }
  /* Bind actions to inputs */
  inputs.registrationCode.onfocus = click.activateInput
  inputs.email.onfocus = click.activateInput
  inputs.password.onfocus = click.activateInput
  inputs.passwordCheck.onfocus = click.activateInput
  /* Bind check to passwords while typing */
  inputs.password.onkeyup = click.confirmValidInput
  inputs.passwordCheck.onkeyup = click.confirmValidInput

  /** Helper Functions :
  * =============================
  * 
  */
  const validation = { 0.5: !0, 0: !0, 1: !0, 2: !0, 3: !0, 4: !0, 5: !0 }

  const buttonBystep = { 0: buttons.startPrep, 0.5: buttons.start, 1: buttons.generate, 2: buttons.plugInDrive, 3: buttons.postDownload, 4: buttons.copied }

  /** 
  * Step Listener to initiate step specific actions
  */
  const constantCheck = () => {
    if (stepTracker === 1) {
      /* Add click listener to page container on Page 2 form intake */
      inlineVariables.contentContainer.onclick =  verifyInputData
    } else if (stepTracker === 2) {
      inlineVariables.downloadFileName.innerHTML = DOWNLOAD_FILE_NAME
    } else if (stepTracker === 3) {
      /* Check for download*/
      verifyDownloadComplete()
    } else if (stepTracker === 4) {
      /* Display back User Email on Page 4 for visual email verification */
      inlineVariables.emailPlaceholder.innerHTML = user.email || console.error('User Email not found. Config may be corrupted.')
    } 
  }

  /**	
   * Validate device by size of screen	
   */	
  const validateScreenSize = () => {	
    /* Detect whether on laptop or desktop */	
    return (window.screen.availWidth >= 768)
  }

  /**	
   * Validate device by confirming non-mobile user agent	
   */	
  const detectMobileUserAgent = () => {	
    /* Detect whether on mobile browser */	
    return (/(android|bb\d+|meego).+mobile|avantgo|bada\/|blackberry|blazer|compal|elaine|fennec|hiptop|iemobile|ip(hone|od)|iris|kindle|lge |maemo|midp|mmp|mobile.+firefox|netfront|opera m(ob|in)i|palm( os)?|phone|p(ixi|re)\/|plucker|pocket|psp|series(4|6)0|symbian|treo|up\.(browser|link)|vodafone|wap|windows ce|xda|xiino/i.test(navigator.userAgent))
  }

  /**
   * Validate if string is email (super simple because actual validation is via sent email)
   * @param {string} email
   */
  const validateEmail = (email) => {
    const re = /[^@]+@[^\.]+\..+/g
    return re.test(String(email).toLowerCase())
  }

  /**
   * Validate if string is valid size
   * @param {string} registrationCode
   */
  const validateRegistrationCode = (registrationCode) => {
    // TODO: define what the min size of the registration code will be
    // TODO: check if the size requirement is met
    return registrationCode !== ""
  }

  /**
   * Update UI to the `step` step
   *
   * @param {int} step
   */
  const updateUiStep = (step) => {
    if (!validation[step]) {
      console.log(`Wrong parameter ${step} in updateUiStep()`)
      return null
    }
    stepTracker = step
    
    constantCheck()
    if(step === 0) {
      return document.body.className = 'step-monitor'
    } else if (step === 0.5) return document.body.className = 'step0b'
    return document.body.className = 'step' + step
  }

 /**
   * Update the progresss bar
   *
   * @param {int} currentTransition
   * @param {bool} rewind
  */
 const updateProgressBar = (currentTransition, rewind = false) => {  
  if (!validation[currentTransition] || currentTransition < 1) {
    console.log(`Wrong parameter ${currentTransition} in updateProgressBar()`)
    return null
  }

  /* Locate current step element and remove 'active' class */
  const childListNodes = document.querySelectorAll('li.progressbar-item')
  const stepIndex = currentTransition - 1
  const currentlyActive = childListNodes[stepIndex]
  currentlyActive.classList.remove('active')

  if (rewind) {
    for (let i=0; i<(stepIndex - 1) + 1; i++) {
      childListNodes[i].classList.add('active')
    }
    return childListNodes[stepIndex - 1]
  }
  else {
    for (let i=0; i<(stepIndex + 1) + 1; i++) {
      childListNodes[i].classList.add('active')
    }
    return childListNodes[stepIndex + 1]
  }
}

  /**
   * Generate save link of hpos-config.json and attach to `button` domElement
   *
   * @param {Object} user
  */
  const generateBlob = user => {
    const configData = config(user.email, user.password, user.registrationCode.trim())
    const configBlob = new Blob([configData.config], { type: 'application/json' })
    
    /* NB: Do not delete!  Keep the below in case we decide to use the HoloPort url it is available right here */
    // console.log('Optional HoloPort url : ', configData.url)
   
    configFileBlob = configBlob
   
    return configFileBlob
  }
  
  /**
   * Verify config was saved before allowing progression to next page
   *
   * @param {Boolean} downloadComplete
  */
  const verifyDownloadComplete = (downloadComplete = downloadTracker, newConfig = resetUserConfig) => {    
    if (downloadComplete) {
      buttons.postDownload.disabled = false
    }
    else if (!downloadComplete && newConfig ) {
      buttons.postDownload.disabled = true
      resetUserConfig = false
      buttons.download.innerHTML = 'Save New Configuration File'
    }
    else return buttons.postDownload.disabled = true
  }

  /**
   * Reset Form Input Feilds while form is active
   *
   * @param {Array} inputElements
  */
  const resetFields = (inputElements) => {    
    for (let inputElement of inputElements) {
      document.querySelector(`#${inputElement.id}-form-item`).classList.remove('error-red')
      inputElement.parentElement.querySelector('.input-item-label').classList.remove('error-red')
      inlineVariables.formErrorMessage.innerHTML = ''
      document.querySelector(`#${inputElement.id}-error-message`).innerHTML = ''
    }
  }

  /**
   * Render specfic form input error messages and styles
   *
   * @param {String} errorMessage
   * @param {Array} errorFieldsArray
  */
  const renderInputError = (errorMessage, errorFieldsArray) => {
    for (let errorField of errorFieldsArray) {    
      document.querySelector(`#${errorField.id}-form-item`).classList.add('error-red')
      errorField.parentElement.querySelector('.input-item-label').classList.add('error-red')

      if (errorMessage === errorMessages.missingFields) inlineVariables.formErrorMessage.innerHTML = errorMessage
      else document.querySelector(`#${errorField.id}-error-message`).innerHTML = errorMessage
    }
    return errorMessage
  }

  
  /**
   * Input form error check
   *
  */
  const confirmValidInput = (submitPressed = signalKeyGen) => {
    const inputElements = Object.values(inputs)
    resetFields(inputElements)
    if(submitPressed) {
      if(!inputs.email.value || !inputs.registrationCode.value) {
        const missingFields = inputElements.filter(inputs => !inputs.value) 
        renderInputError(errorMessages.missingFields, missingFields)
      } else if (!validateEmail(inputs.email.value)) {
        renderInputError(errorMessages.email, [inputs.email])
      } else if (!validateRegistrationCode(inputs.registrationCode.value)) {
        renderInputError(errorMessages.registrationCode, [inputs.registrationCode])
      } 
      else if (!inputs.password.value || inputs.password.value.length <= 7) {
        renderInputError(errorMessages.password, [inputs.password])
      } else if (inputs.password.value && inputs.password.value !== inputs.passwordCheck.value) {
        const errorInputs = [inputs.passwordCheck]
        renderInputError(errorMessages.passwordCheck, errorInputs)
      } else return true
    } else if (inputs.password.value && inputs.passwordCheck.value && inputs.password.value !== inputs.passwordCheck.value) {
      const errorInputs = [inputs.passwordCheck]
      renderInputError(errorMessages.passwordCheck, errorInputs)
    } else if (inputs.password.value && inputs.password.value.length <= 7) {
      renderInputError(errorMessages.password, [inputs.password])
    } else return true

    return false
  }

  /**
   * Verify all form input before allowing progression to next page
  */
  const verifyInputData = () => {
    let inputValidity = confirmValidInput()
    if (inputValidity) buttons.generate.disabled = false
    else buttons.generate.disabled = true
    return inputValidity
  }
})()
