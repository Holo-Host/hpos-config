(async () => {
  const { config } = await import('../pkg')

  const DOWNLOAD_FILE_NAME = 'hpos-config.json'

  let stepTracker
  let signalKeyGen
  let downloadTracker
  let resetUserConfig

  /* Parse HTML elements */
  const buttons = {
    start: document.querySelector('#start-button'),
    generate: document.querySelector('#generate-button'),
    download: document.querySelector('#download-button'),
    postDownload: document.querySelector('#post-download-button'),
    copied: document.querySelector('#copied-button'),
    openOlay: document.querySelector('#open-overlay'),
    closeOlay: document.querySelector('#close-overlay'),
    back1: document.querySelector('#back-button1'),
    back2: document.querySelector('#back-button2'),
    back3: document.querySelector('#back-button3'),
    back4: document.querySelector('#back-button4')
  }

  const inputs = {
    email: document.querySelector('#email'),
    password: document.querySelector('#password'),
    passwordCheck: document.querySelector('#password-check'),
  }

  const inlineVariables = {
    contentContainer: document.querySelector('#content-container'),
    emailPlaceholder: document.querySelector('#email-placeholder'),
    emailInputArea: document.querySelector('#email-form-item'),
    passwordInputArea: document.querySelector('#password-form-item'),
    passwordCheckInputArea: document.querySelector('#password-check-form-item'),
    holoportFlyingBookend: document.querySelector('#holoport-flying-bookend'),
    formSuccessMessage: document.querySelector('#form-success-message'),
    formErrorMessage: document.querySelector('#form-error-message'),
    timerMessage: document.querySelector('#timer-sub-text')
  }

  const errorMessages = {
    missingFields: '*Please complete missing fields.',
    email: '*Email domain not recognized',
    password: '*Your password needs to be at least eight character in length',
    passwordCheck: '*Passwords do not match',
    generateConfig: '*An error occured when configuring your user file. Please update your information and try again.'
  }

  const user = {
    email: '',
    password: ''
  }


  /** Actions executed at button click
  * ======================================
  */
  const click = {
    start: () => {
      if (!validateBrowser()) {
        alert('Please upgrade your browser to newer version.')
        return null
      }
      // TODO: RESET TO BELOW ONCE OUT OF DEV MODE
      updateUiStep(1)
  
      // DEV MODE HACK TO SWITCH THROUGH PAGES
      // updateUiStep(2)
    },  
    generate: async () => {
      signalKeyGen = true
      const inputValidity = await verifyInputData()
      if (!inputValidity) return buttons.generate.disabled = true

      /* Set user config */
      user.email = inputs.email.value
      user.password = inputs.password.value
      console.log('user config : ', user)

      /* Communicate visually that something is happening in the bkgd */
      buttons.generate.disabled = true
      downloadTracker = false
      click.openLoader()

      setTimeout(() => {
        // Generate hpos-config.json and create download blob attached to url
        try {
          inlineVariables.formErrorMessage.innerHTML = ''
          generateDownload(user, buttons.download)
        } catch (e) {
          console.log(`Error executing generateDownload with an error ${e}`)
          inlineVariables.formErrorMessage.innerHTML = errorMessages.generateConfig
          return null
        }

        /* Placeholder: Visually communicate that config generated in the bkgd : */
        // inlineVariables.formSuccessMessage.innerText = 'Your User File has been successfully generated.'

        /* Clean State */
        buttons.generate.disabled = false
        click.closeLoader()
        updateUiStep(2)
        updateProgressBar(1)
      }, 1500)
    },
    download: () => {
      /* Communicate visually that something is happening in the bkgd */
      buttons.download.disabled = true
      buttons.download.innerText = 'Downloading File'

      setTimeout(() => {
        /* Clean State */
        buttons.download.disabled = false
        buttons.download.innerText = 'Download File Again'
        downloadTracker = true
        verifyDownloadComplete(downloadTracker)
      }, 1000)
    },
    postDownload: () => {  
      updateUiStep(3)
      updateProgressBar(2)
    },
    copied: () => {
      updateUiStep(4)
      updateProgressBar(3)
    },
    openOlay: () => {
      document.querySelector('#fixed-overlay-tos').style.display = 'block'
      document.querySelector('#modal-overlay').style.display = 'block'
    },
    closeOlay: () => {
      document.querySelector('#fixed-overlay-tos').style.display = 'none'
      document.querySelector('#modal-overlay').style.display = 'none'
    },
    openLoader: () => {
      document.querySelector('#fixed-overlay-loader').style.display = 'block'
      document.querySelector('#modal-overlay-loader').style.display = 'block'
    },
    closeLoader: () => {
      document.querySelector('#fixed-overlay-loader').style.display = 'none'
      document.querySelector('#modal-overlay-loader').style.display = 'none'
    },
    back1: () => {
      updateUiStep(0)
    },
    back2: () => {
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
  buttons.start.onclick = click.start
  buttons.generate.onclick = click.generate
  buttons.download.onclick = click.download
  buttons.postDownload.onclick = click.postDownload
  buttons.copied.onclick = click.copied
  // buttons.finalStage.onclick = click.finalStage
  buttons.openOlay.onclick = click.openOlay
  buttons.closeOlay.onclick = click.closeOlay
  buttons.back1.onclick = click.back1
  buttons.back2.onclick = click.back2
  buttons.back3.onclick = click.back3
  buttons.back4.onclick = click.back4
  document.onkeyup = click.activateInput
  /* Bind input actions to inputArea actions */
  inlineVariables.emailInputArea.onclick = e => { inputs.email.focus(); return click.activateInput(e) }
  inlineVariables.passwordInputArea.onclick = e => { inputs.password.focus(); return click.activateInput(e) }
  inlineVariables.passwordCheckInputArea.onclick = e => { inputs.passwordCheck.focus(); return click.activateInput(e) }
  /* Bind actions to inputs */
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
  const validation = { 0: !0, 1: !0, 2: !0, 3: !0, 4: !0 }

  const buttonBystep = { 0: buttons.start, 1: buttons.generate, 2: buttons.postDownload, 3: buttons.copied, 4: buttons.finalStage }

  const addMinutesToDateTime = (dt, minutes) => new Date(dt.getTime() + minutes*60000)


  /** 
  * Step Listener to initiate step specific actions
  */
  const constantCheck = () => {
    if (stepTracker === 1) {
      /* Add click listener to page container on Page 2 form intake */
      inlineVariables.contentContainer.onclick =  verifyInputData
    } else if (stepTracker === 4) {
      /* Display back User Email on Page 4 for visual email verification */
      inlineVariables.emailPlaceholder.innerHTML = user.email || console.error('User Email not found. Config may be corrupted.')
    } else if (stepTracker === 2) {
      /* Check for download*/
      verifyDownloadComplete()
    }
  }

  /**
   * Validate if string is email (super simple because actual validation is via sent email)
   * @param {string} email
   */
  const validateEmail = (email) => {
    const re = /^\S+@\S+$/
    return re.test(String(email).toLowerCase())
  }

  /**
   * Validate if browser supports required functions
   */
  const validateBrowser = () => {
    /* Detect if browser supports download attribute on <a> */
    return (typeof buttons.download.download !== 'undefined')
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
    }
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
   * Generate download link of hpos-config.json and attach to `button` domElement
   *
   * @param {Object} user
   * @param {DomElement} button - a DomElement that will have download and attribute props updated
  */
  const generateDownload = (user, button) => {
    console.log('Generating User Keys and creating Config...')
    const configData = config(user.email, user.password)
    const configBlob = new Blob([configData.config], { type: 'application/json' })
    const url = URL.createObjectURL(configBlob)

    if (button.nodeName !== 'A') throw new Error('Download button has to be node <a> type')

    button.href = url
    button.download = DOWNLOAD_FILE_NAME

    /* In case we decide to use the HoloPort url it is available right here */
    console.log('Optional HoloPort url : ', configData.url)

    return url
  }

  /**
   * Verify config was downloaded before allowing progression to next page
   *
   * @param {Boolean} downloadComplete
  */
  const verifyDownloadComplete = (downloadComplete = downloadTracker, newConfig = resetUserConfig) => {
    if (downloadComplete) {
      buttons.postDownload.disabled = false
    }
    else if (!downloadComplete && newConfig ) {
      buttons.postDownload.disabled = true
      buttons.download.innerText = 'Download Updated File'
      resetUserConfig = false
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
      if(!inputs.email.value) {
        const missingFields = inputElements.filter(inputs => !inputs.value) 
        renderInputError(errorMessages.missingFields, missingFields)
      } else if (!validateEmail(inputs.email.value)) {
        renderInputError(errorMessages.email, [inputs.email])
      } else if (!inputs.password.value || inputs.password.value.length <= 7) {
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
