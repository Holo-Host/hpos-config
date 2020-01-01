
(async () => {
  const filesaver = require('file-saver');
  const { config } = await import('../pkg')

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
    formErrorMessage: document.querySelector('#form-error-message'),
    downloadFileName: document.querySelector('#download-file')
  }

  const errorMessages = {
    missingFields: 'Please complete missing fields.',
    email: 'Email domain not recognized',
    password: 'Your password needs to be at least eight character in length',
    passwordCheck: 'Passwords do not match',
    generateConfig: 'An error occured when configuring your user file. Please update your information and try again.'
  }

  const user = {
    email: '',
    password: ''
  }


  /** Actions executed at button click
  * ======================================
  */
  const click = {
    startPrep: () => {
      if (!validateScreenSize() || detectMobileUserAgent()) {
        console.log('!validateScreenSize() :', !validateScreenSize())
        console.log('detectMobileUserAgent() : ', detectMobileUserAgent())
        
        alert('Please visit this site on a desktop or laptop computer to continue.')
        return null
      }

      // TODO: RESET TO BELOW ONCE OUT OF DEV MODE
      updateUiStep(0.5)

      // DEV MODE HACK TO SWITCH THROUGH PAGES
      // updateUiStep(4)
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
      user.email = inputs.email.value
      user.password = inputs.password.value

      // DEV MODE - Config Check: 
      // console.log('user config : ', user)
      
      /* Communicate visually that something is happening in the bkgd */
      buttons.generate.disabled = true
      downloadTracker = false
      click.openLoader()

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
    return (window.screen.availWidth >= 1024 && window.screen.availHeight >= 768)
  }

  /**	
   * Validate device by confirming non-mobile user agent	
   */	
  const detectMobileUserAgent = () => {	
    /* Detect whether on mobile browser */	
    return (/(android|bb\d+|meego).+mobile|avantgo|bada\/|blackberry|blazer|compal|elaine|fennec|hiptop|iemobile|ip(hone|od)|iris|kindle|lge |maemo|midp|mmp|mobile.+firefox|netfront|opera m(ob|in)i|palm( os)?|phone|p(ixi|re)\/|plucker|pocket|psp|series(4|6)0|symbian|treo|up\.(browser|link)|vodafone|wap|windows ce|xda|xiino/i.test(navigator.userAgent)||/1207|6310|6590|3gso|4thp|50[1-6]i|770s|802s|a wa|abac|ac(er|oo|s\-)|ai(ko|rn)|al(av|ca|co)|amoi|an(ex|ny|yw)|aptu|ar(ch|go)|as(te|us)|attw|au(di|\-m|r |s )|avan|be(ck|ll|nq)|bi(lb|rd)|bl(ac|az)|br(e|v)w|bumb|bw\-(n|u)|c55\/|capi|ccwa|cdm\-|cell|chtm|cldc|cmd\-|co(mp|nd)|craw|da(it|ll|ng)|dbte|dc\-s|devi|dica|dmob|do(c|p)o|ds(12|\-d)|el(49|ai)|em(l2|ul)|er(ic|k0)|esl8|ez([4-7]0|os|wa|ze)|fetc|fly(\-|_)|g1 u|g560|gene|gf\-5|g\-mo|go(\.w|od)|gr(ad|un)|haie|hcit|hd\-(m|p|t)|hei\-|hi(pt|ta)|hp( i|ip)|hs\-c|ht(c(\-| |_|a|g|p|s|t)|tp)|hu(aw|tc)|i\-(20|go|ma)|i230|iac( |\-|\/)|ibro|idea|ig01|ikom|im1k|inno|ipaq|iris|ja(t|v)a|jbro|jemu|jigs|kddi|keji|kgt( |\/)|klon|kpt |kwc\-|kyo(c|k)|le(no|xi)|lg( g|\/(k|l|u)|50|54|\-[a-w])|libw|lynx|m1\-w|m3ga|m50\/|ma(te|ui|xo)|mc(01|21|ca)|m\-cr|me(rc|ri)|mi(o8|oa|ts)|mmef|mo(01|02|bi|de|do|t(\-| |o|v)|zz)|mt(50|p1|v )|mwbp|mywa|n10[0-2]|n20[2-3]|n30(0|2)|n50(0|2|5)|n7(0(0|1)|10)|ne((c|m)\-|on|tf|wf|wg|wt)|nok(6|i)|nzph|o2im|op(ti|wv)|oran|owg1|p800|pan(a|d|t)|pdxg|pg(13|\-([1-8]|c))|phil|pire|pl(ay|uc)|pn\-2|po(ck|rt|se)|prox|psio|pt\-g|qa\-a|qc(07|12|21|32|60|\-[2-7]|i\-)|qtek|r380|r600|raks|rim9|ro(ve|zo)|s55\/|sa(ge|ma|mm|ms|ny|va)|sc(01|h\-|oo|p\-)|sdk\/|se(c(\-|0|1)|47|mc|nd|ri)|sgh\-|shar|sie(\-|m)|sk\-0|sl(45|id)|sm(al|ar|b3|it|t5)|so(ft|ny)|sp(01|h\-|v\-|v )|sy(01|mb)|t2(18|50)|t6(00|10|18)|ta(gt|lk)|tcl\-|tdg\-|tel(i|m)|tim\-|t\-mo|to(pl|sh)|ts(70|m\-|m3|m5)|tx\-9|up(\.b|g1|si)|utst|v400|v750|veri|vi(rg|te)|vk(40|5[0-3]|\-v)|vm40|voda|vulc|vx(52|53|60|61|70|80|81|83|85|98)|w3c(\-| )|webc|whit|wi(g |nc|nw)|wmlb|wonu|x700|yas\-|your|zeto|zte\-/i.test(navigator.userAgent.substr(0,4)))
  }

  /**
   * Validate if string is email (super simple because actual validation is via sent email)
   * @param {string} email
   */
  const validateEmail = (email) => {
    // const re = /^\S+@\S+$/
    const re = /[^@]+@[^\.]+\..+/g
    return re.test(String(email).toLowerCase())
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
    const configData = config(user.email, user.password)
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
