import './style.css'

(async () => {
  const { config } = await import('./pkg')

  const DOWNLOAD_FILE_NAME = 'hpos-state.json'

  let stepTracker
  let configDownloaded

  // Parse UI elements
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
    back3: document.querySelector('#back-button3')
  }

  const inputs = {
    email: document.querySelector('#email'),
    password: document.querySelector('#password'),
    passwordCheck: document.querySelector('#password-check'),
    deviceName: document.querySelector('#device-name')
  }

  const user = {
    email: '',
    password: '',
    device_name: ''
  }

  const inlineVariables = {
    emailPlaceholder: document.querySelector('#email-placeholder'),
    emailLabel: document.querySelector('#email-label'),
    passwordLabel: document.querySelector('#password-label'),
    passwordCheckLabel: document.querySelector('#password-check-label'),
    deviceNameLabel: document.querySelector('#device-name-label'),
    holoportFlyingBookend: document.querySelector('#holoport-flying-bookend')
  }

  const errorMessages = {
    missingFields: '*Please complete missing fields.',
    email: 'Email domain not recognized',
    password: 'Password cannot be empty',
    passwordCheck: 'Your password needs to be at least eight characters in length'
  }

  // Actions executed at button click
  // =============================
  const click = {
    start: () => {
      if (!validateBrowser()) {
        alert('Please upgrade your browser to newer version.')
        return null
      }
      updateUiStep(1)
    },
    generate: () => {
      // Set user config
      user.email = inputs.email.value
      user.password = inputs.password.value
      user.device_name = inputs.deviceName.value
      console.log('user config : ', user) 

      // Communicate visually that something is happening in the bkgd
      buttons.generate.disabled = true
      buttons.generate.innerText = 'Generating Keys...'

      // Move generateDownload out of exec flow
      setTimeout(() => {
        // Generate hpos-state.json and create download blob attached to url
        try {
          generateDownload(user, buttons.download)
        } catch (e) {
          console.log(`Error executing generateDownload with an error ${e}`)
          return null
        }

        // Clean State
        buttons.generate.disabled = false
        buttons.generate.innerText = 'Generate'
        updateUiStep(2)
        updateProgressBar(1)
      }, 50)
    },
    download: () => {
      // Communicate visually that something is happening in the bkgd
      buttons.download.disabled = true
      buttons.download.innerText = 'Saving to USB Drive...'
      // Update user email in the UI
      document.querySelector('#email-placeholder').innerText = user.email

      // Clean State
      setTimeout(() => {
        buttons.download.disabled = false
        buttons.download.innerText = 'Save to USB Drive'
        configDownloaded = true
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
    },
    closeOlay: () => {
      document.querySelector('#fixed-overlay-tos').style.display = 'none'
    },
    back1: () => {
      const rewind = true
      updateProgressBar(1, rewind)
      updateUiStep(0)
    },
    back2: () => {
      const rewind = true
      updateProgressBar(2, rewind)
      updateUiStep(1)
    },
    back3: () => {
      const rewind = true
      updateProgressBar(3, rewind)
      updateUiStep(2)
    },
    activateInput: event => {
      const inputId = event.target.id
      const labelId = document.querySelector(`#${inputId}-label`)
      const activeInputs = document.querySelectorAll('.input-active')
      if (activeInputs) {
        for (let activeInput of activeInputs) {
          if (!activeInput.parentElement.querySelector('input').value){
            activeInput.classList.remove('input-active')
            activeInput.dataset.shrink = 'false'
          }
        }
      }
      labelId.classList.add('input-active')
      labelId.dataset.shrink = 'true'
      
      verifyInputData()
    }
  }

  // Set intial state for all config actions buttons to 'disabled'
  buttons.generate.disabled = true
  buttons.postDownload.disabled = true

  // Bind actions to buttons
  buttons.start.onclick = click.start
  buttons.generate.onclick = click.generate
  buttons.download.onclick = click.download
  buttons.postDownload.onClick = click.postDownload
  buttons.copied.onclick = click.copied
  buttons.openOlay.onclick = click.openOlay
  buttons.closeOlay.onclick = click.closeOlay
  buttons.back1.onclick = click.back1
  buttons.back2.onclick = click.back2
  buttons.back3.onclick = click.back3
  
  // Bind listeners to inputs
  inlineVariables.deviceNameLabel.onclick = inputs.deviceName.click()
  inlineVariables.emailLabel.onclick = inputs.email.click()
  inlineVariables.passwordLabel.onclick = inputs.password.click()
  inlineVariables.passwordCheckLabel.onclick = inputs.passwordCheck.click()

  inputs.deviceName.onfocus = click.activateInput
  inputs.email.onfocus = click.activateInput
  inputs.password.onfocus = click.activateInput
  inputs.passwordCheck.onfocus = click.activateInput
  // TODO: Update the password inputs to haave passive event listeners...
  // inputs.password.addEventListener('focus', click.activateInput, true) = click.activateInput
  // inputs.passwordCheck.addEventListener('focus', click.activateInput, true) = click.activateInput



  // Helper Functions :
  // =============================

  // Email Verification - Display back User Email on Step 4
  inlineVariables.emailPlaceholder.innerHTML = user.email || 'email@placeholder.com'

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
    // Detect if browser supports download attribute on <a>
    return (typeof buttons.download.download !== 'undefined')
  }

  /**
   * Update UI to the `step` step
   *
   * @param {int} step
   */
  // 
  const validation = { 0: !0, 1: !0, 2: !0, 3: !0, 4: !0 }
  // 
  const updateUiStep = (step) => {
    if (!validation[step]) {
      console.log(`Wrong parameter ${step} in updateUiStep()`)
      return null
    }
    stepTracker = step
    renderStyle(stepTracker)
    return document.body.className = 'step' + step
  }

  /**
   * Update the progresss bar
   * @param {int} currentTransition
   * @param {bool} rewind
   */
  const updateProgressBar = (currentTransition, rewind = false) => {
    if (currentTransition <= 1) rewind = false
    if (!validation[currentTransition]) {
      console.log(`Wrong parameter ${currentTransition} in updateProgressBar()`)
      return null
    }
    const stepIndex = currentTransition - 1

    // Locate current step element and remove 'active' class
    const childListNodes = document.querySelectorAll('li.progressbar-item')
    const currentlyActive = childListNodes[stepIndex]
    currentlyActive.classList.remove('active')

    if (rewind) {
      return childListNodes[stepIndex - 1].classList.add('active')
    }
    else childListNodes[stepIndex + 1].classList.add('active')
  }


    /**
   * Generate download link of holo-state.json and attach to `button` domElement
   *
   * @param {Object} user
   * @param {DomElement} button - a DomElement that will have download and attribute props updated
   */
  const generateDownload = (user, button) => {
    console.log('generating keys...')
    const configData = config(user.email, user.password, user.device_name)
    const configBlob = new Blob([configData.config], { type: 'application/json' })
    const url = URL.createObjectURL(configBlob)

    if (button.nodeName !== 'A') throw new Error('Download button has to be node <a> type')

    button.href = url
    button.download = DOWNLOAD_FILE_NAME

    // In case we decide to use the HoloPort url it is available right here
    console.log('Optional HoloPort url : ', configData.url)
  }

  const resetFields = (inputElements) => {
    for (let inputElement of inputElements) { 
      inputElement.parentElement.querySelector('.form-item').classList.remove('error')
      inputElement.parentElement.querySelector('.input-item-label').classList.remove('error')
      inputElement.parentElement.parentElement.querySelector(`#${inputElement.id}-error-message`).innerHTML = ''
    }
  }

  const renderInputError = (errorMessage, errorFieldsArray) => {
    console.log('errorMessage', errorMessage)
    console.log('errorFieldsArray', errorFieldsArray)
    for (let errorField of errorFieldsArray) {      
      errorField.parentElement.querySelector('.form-item').classList.add('error')
      errorField.parentElement.querySelector('.input-item-label').classList.add('error')
      
      console.log('FORM-ERROR', document.querySelector('#form-error')) 

      if(errorMessage === errorMessages.missingFields) document.querySelector('#form-error').innerHTML = errorMessage
      else errorField.parentElement.parentElement.querySelector(`#${errorField.id}-error-message`).innerHTML = errorMessage
    }
    return errorMessage
  }

  const confirmValidInput = () => {
    const inputElements = Object.values(inputs)
    resetFields(inputElements)
    if(!inputs.deviceName.value || !inputs.email.value || !inputs.password.value || !inputs.passwordCheck.value) {
    const missingFields = inputElements.filter(inputElement => !inputElement.value) 
    renderInputError(errorMessages.missingFields, missingFields)
      return false
    } else if (!validateEmail(inputs.email.value)) {
      renderInputError(errorMessages.email, [inputs.email])
      return false
    } else if (!inputs.password.value) {
      renderInputError(errorMessages.password, [inputs.password])
      return false
    } else if (inputs.password.value !== inputs.passwordCheck.value) {
      const errorInputs = [inputs.password.value, inputs.passwordCheck.value]
      renderInputError(errorMessages.passwordCheck, errorInputs)
      return false
    } else {
      return true
    }
  }


  // Step/Page Specific Functions :
  // =============================

  // All page 2 inputs have values
  const verifyInputData = () => {
    let inputValidity = confirmValidInput()
    if(inputs.deviceName.value && inputs.email.value && inputs.password.value && inputs.passwordCheck.value) {
      if(inputValidity) buttons.generate.disabled = false
      else {
        inputValidity = false
        buttons.generate.disabled = true
      }
    }
    console.log('inputValidity ... ', inputValidity)
    return inputValidity
  }

  // Page 3 config download complete 
  const verifyDownloadComplete = () => {
    if(configDownloaded) {
      console.log('CONFIG IS DOWNLOADED ... ')
      buttons.postDownload.disabled = false
    }
    return configDownloaded
  }
  verifyDownloadComplete()

  // Add styles based on step/page
  const renderStyle = (step = stepTracker = 0) => {
    if (step === 1 || step === 2 || step === 3){
      console.log('RENDERSTYLE : STEP should equal 1, 2 or 3 >>>> : ', step)
      inlineVariables.holoportFlyingBookend.style.zIndex = '2'
    }

    console.log('Current step in renderStyle : ', step)
    return step
  }
  renderStyle(stepTracker)
})()
