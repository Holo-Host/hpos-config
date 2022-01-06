
(async () => {
  const filesaver = require('file-saver');
  const hcSeedBundle = require('@holochain/hc-seed-bundle');
  const { config } = await import('../pkg')
  const {
    validateRegistrationCode,
    validateScreenSize,
    detectMobileUserAgent,
    validateEmail,
    validatePassphrae } = await import('./validation')
  const { genConfigFileName, toBase64 } = await import('./utils')
  const SEED_FILE_NAME = 'master-seed'
  let stepTracker = 0
  let signalKeyGen = false
  let resetUserConfig = false
  let downloadConfigTracker = false
  let downloadSeedTracker = false
  let configFileBlob = ''
  let master
  let deviceNumber = 0
  let deviceID
  let registrationCode

  /* Parse HTML elements */
  const buttons = {
    nextStep: document.querySelector('#next-button'),
    prevStep: document.querySelector('#previous-button'),
    genSeed: document.querySelector('#gen-seed-button'),
    download: document.querySelector('#download-button'),
    closeNotice: document.querySelector('#close-notice'),
    back3Confirmation: document.querySelector('#back-button3-confirmation'),
    exit: document.querySelector('#exit-button'),
    loop: document.querySelector('#loop-button'),
    closeModalIntro: document.querySelector('#close-modal-intro'),
    closeModalOutro: document.querySelector('#close-modal-outro')
  }

  const inputs = {
    registrationCode: document.querySelector('#registration-code'),
    seedPassphrase: document.querySelector('#seed-passphrase'),
    email: document.querySelector('#email'),
    password: document.querySelector('#password'),
    passwordCheck: document.querySelector('#password-check'),
  }

  const inlineVariables = {
    contentContainer: document.querySelector('#content-container'),
    registrationCodeInputArea: document.querySelector('#registration-code-form-item'),
    seedPassphraseInputArea: document.querySelector('#seed-passphrase-form-item'),
    emailPlaceholder: document.querySelector('#email-placeholder'),
    emailInputArea: document.querySelector('#email-form-item'),
    passwordInputArea: document.querySelector('#password-form-item'),
    passwordCheckInputArea: document.querySelector('#password-check-form-item'),
    formErrorMessage: document.querySelector('#form-error-message'),
    downloadFileName: document.querySelector('#download-file'),
    currentHoloportDescriptor: document.querySelector('#current-holoport-descriptor')
  }

  const errorMessages = {
    missingFields: 'Please complete missing fields.',
    seedPassphrase: 'Your passphrase needs to be at least 20 character in length',
    registrationCode: 'Invalid code',
    email: 'Email domain not recognized',
    password: 'Your password needs to be at least eight character in length',
    passwordCheck: 'Passwords do not match',
    generateConfig: 'An error occurred when configuring your user file. Please update your information and try again.'
  }

  const user = {
    registrationCode: '',
    email: '',
    password: ''
  }

  // global variable used to pass seed passphrase between steps 2 and 3
  let seedPassphrase

  /** Actions executed at button click
  * ======================================
  */
  const click = {
    nextStep: () => {
      switch (stepTracker) {
        case 0:
          if (!validateScreenSize() || detectMobileUserAgent()) {
            const confirmed = confirm('This experience has not been optimized for mobile devices. Please continue only if you are using a laptop or PC.\n\nContinuing on a mobile device may result in unexpected issues.')
            if (confirmed === true) return updateUiStep(1)
            else return null
          } else {
            updateUiStep(0.5)
          }
          break
        case 0.5:
          updateUiStep(1)
          inputs.email.click()
          break
        case 1:
          if (!verifyInputData()) return buttons.nextStep.disabled = true
          // Load registration Code for use in later steps
          registrationCode = inputs.registrationCode.value
          verifySeedDownloadComplete()
          updateUiStep(2)
          updateProgressBar(1)
          click.showModalSeedIntro()
          break
        case 2:
          if (!verifyInputData()) {
            buttons.nextStep.disabled = true
            return
          }
          seedPassphrase = inputs.seedPassphrase.value
          updateUiStep(3)
          updateProgressBar(2)
          break
        case 3:
          updateUiStep(4)
          updateProgressBar(3)
          click.showModalSeedOutro()
          break
        case 4:
          generate()
          break
        case 5:
          updateUiStep(6)
          updateProgressBar(5)
          break
        case 6:
          updateUiStep(7)
          break
        default:
          throw new Error(`unexpected stepTracker in nextStep: ${stepTracker}`)
      }
    },
    prevStep: () => {
      switch (stepTracker) {
        case 0.5:
          updateUiStep(0)
          break
        case 1:
          updateUiStep(0.5)
          break
        case 2:
        case 4:
        case 5:
        case 6:
          const rewind = true
          updateProgressBar(stepTracker, rewind)
          updateUiStep(stepTracker - 1)
          break
        case 3:
          click.openNotice()
          break
        default:
          throw new Error(`unexpected stepTracker in prevStep: ${stepTracker}`)
      }
    },
    genSeed: async () => {
      // Load registration Code for use in later steps
      /* Communicate visually that something is happening in the bkgd */
      buttons.genSeed.classList.add('disabled')
      buttons.genSeed.disabled = true
      buttons.genSeed.innerHTML = 'Saving Master Seed...'

      setTimeout(async () => {
        try {
          // setup bundler
          await hcSeedBundle.seedBundleReady
          // generate a new pure entropy master seed
          // Note: we will clear the secret at exit of this app
          master = hcSeedBundle.UnlockedSeedBundle.newRandom({
            bundleType: 'master'
          })
          master.setAppData({
            generate_by: "quickstart-v2.0"
          })
          // we need the passphrase as a Uint8Array

          const pw = (new TextEncoder()).encode(seedPassphrase)

          // clear passphrase from memory
          seedPassphrase = null

          const encodedBytes = master.lock([
            new hcSeedBundle.SeedCipherPwHash(
              hcSeedBundle.parseSecret(pw), 'minimum')
          ])

          // DEV MODE - check pub key for devices:
          console.log("Created master seed: ", master.signPubKey);

          const seedBlob = new Blob([toBase64(encodedBytes)], { type: 'text/plain' })
          filesaver.saveAs(seedBlob, SEED_FILE_NAME)

        } catch (e) {
          throw new Error(`Error saving config. Error: ${e}`)
        }

        /* Clean State */
        downloadSeedTracker = true
        buttons.genSeed.disabled = true
        buttons.genSeed.innerHTML = 'Saved Master Seed*'
        verifySeedDownloadComplete(downloadSeedTracker)
      }, 1000)
    },
    download: async () => {
      /* Communicate visually that something is happening in the bkgd */
      buttons.download.disabled = true
      buttons.download.innerHTML = 'Saving Configuration File...'

      setTimeout(() => {
        try {
          filesaver.saveAs(configFileBlob, genConfigFileName(deviceNumber, deviceID))
        } catch (e) {
          // TODO do throw here throw new Error(`Error saving config. Error: ${e}`)
        }

        /* Clean State */
        downloadConfigTracker = true
        buttons.download.disabled = false
        buttons.download.innerHTML = 'Save Configuration File Again'
        verifyDownloadComplete(downloadConfigTracker)
      }, 1000)
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
    showModalSeedIntro: () => {
      document.querySelector('#modal-seed-intro').style.display = 'block'
    },
    showModalSeedOutro: () => {
      document.querySelector('#modal-seed-outro').style.display = 'block'
    },
    closeSeedModals: () => {
      document.querySelector('#modal-seed-intro').style.display = 'none'
      document.querySelector('#modal-seed-outro').style.display = 'none'
    },
    back3Confirmation: () => {
      click.closeNotice()
      // Reseting UI
      const rewind = true
      signalKeyGen = false
      resetUserConfig = true
      downloadConfigTracker = false
      downloadSeedTracker = false
      configFileBlob = ''
      master = undefined
      deviceNumber = 0
      deviceID = undefined
      updateProgressBar(3, rewind)
      updateProgressBar(2, rewind)
      updateUiStep(1)
    },
    exit: () => {
      // clear our secrets
      master && master.zero()
      updateUiStep(-1)
    },
    loop: () => {
      deviceNumber++
      inlineVariables.currentHoloportDescriptor.innerHTML = 'additional'
      // hide back option
      buttons.back4.setAttribute("hidden", "hidden");
      updateProgressBar(6, true)
      updateProgressBar(5, true)
      updateUiStep(4)
    },
    forumHelp: e => {
      e.preventDefault()
      window.open(
        'https://forum.holo.host',
        '_blank'
      )
    },
    handleKeyPress: event => {
      const keycode = (event.keyCode ? event.keyCode : event.which)
      /* Number 13 is the "Enter" key on the keyboard */
      if (keycode === 13 && stepTracker <= 4) {
        console.log('preventing default')
        event.preventDefault()
      }
      else return null
    },
    handleKeyUp: event => {
      const keycode = (event.keyCode ? event.keyCode : event.which)
      /* Number 13 is the "Enter" key on the keyboard */
      if (keycode === 13 && stepTracker <= 4) {
        event.preventDefault()
        click.nextStep()
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
          if (!activeInput.parentElement.querySelector('input').value) {
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
  document.querySelector('body').onkeypress = click.handleKeyPress
  document.querySelector('body').onkeyup = click.handleKeyUp

  /* Bind actions to buttons */
  buttons.nextStep.onclick = click.nextStep
  buttons.prevStep.onclick = click.prevStep
  buttons.genSeed.onclick = click.genSeed
  buttons.closeNotice.onclick = click.closeNotice
  buttons.back3Confirmation.onclick = click.back3Confirmation
  buttons.exit.onclick = click.exit
  buttons.loop.onclick = click.loop
  buttons.closeModalIntro.onclick = click.closeSeedModals
  buttons.closeModalOutro.onclick = click.closeSeedModals


  // buttons.forumHelp.onclick = click.forumHelp
  document.onkeyup = click.activateInput
  /* Bind input actions to inputArea actions */
  inlineVariables.registrationCodeInputArea.onclick = e => { inputs.registrationCode.focus(); return click.activateInput(e) }
  inlineVariables.seedPassphraseInputArea.onclick = e => { inputs.seedPassphrase.focus(); return click.activateInput(e) }
  inlineVariables.emailInputArea.onclick = e => { inputs.email.focus(); return click.activateInput(e) }
  inlineVariables.passwordInputArea.onclick = e => { inputs.password.focus(); return click.activateInput(e) }
  inlineVariables.passwordCheckInputArea.onclick = e => { inputs.passwordCheck.focus(); return click.activateInput(e) }
  /* Bind actions to inputs */
  inputs.registrationCode.onfocus = click.activateInput
  inputs.seedPassphrase.onfocus = click.activateInput
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
  const validation = { 0.5: !0, 0: !0, 1: !0, 2: !0, 3: !0, 4: !0, 5: !0, 6: !0, 7: !0, '-1': !0 }

  /**
  * Step Listener to initiate step specific actions
  */
  const constantCheck = () => {
    if (stepTracker === 1) {
      /* Add click listener to page container on Page 2 form intake */
      inlineVariables.contentContainer.onclick = verifyInputData
    } else if (stepTracker === 2) {
      verifyDownloadComplete()
    } else if (stepTracker === 3) {
      /* Check for download*/
      verifyDownloadComplete()
    } else if (stepTracker === 5) {
      // TODO put this back inlineVariables.downloadFileName.innerHTML = genConfigFileName(deviceNumber, deviceID)
      inlineVariables.downloadFileName.innerHTML = "whatever.jsoon"
    }
  }
  /**
   * Update UI to the `step` step
   *
   * @param {int} step
   */
  const updateUiStep = (step) => {
    console.log('updateUiStep', step)
    if (!validation[step]) {
      console.log(`Wrong parameter ${step} in updateUiStep()`)
      return null
    }
    stepTracker = step
    buttons.nextStep.disabled = false;
    constantCheck()

    switch (step) {
      case 0:
        document.body.className = 'step0'
        break
      case 0.5:
        document.body.className = 'step1a'
        break
      case -1:
        document.body.className = 'step-exit'
        break
      default:
        document.body.className = 'step' + step
    }
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
      for (let i = 0; i < (stepIndex - 1) + 1; i++) {
        childListNodes[i].classList.add('active')
      }
      return childListNodes[stepIndex - 1]
    }
    else {
      for (let i = 0; i < (stepIndex + 1) + 1; i++) {
        childListNodes[i].classList.add('active')
      }
      return childListNodes[stepIndex + 1]
    }
  }

  const generate = async () => {
    signalKeyGen = true
    const inputValidity = await verifyInputData()
    if (!inputValidity) return buttons.nextStep.disabled = true

    /* Set user config */
    user.registrationCode = inputs.registrationCode.value
    user.email = inputs.email.value
    user.password = inputs.password.value

    // DEV MODE - Config Check:
    // console.log('user config : ', user)

    /* Communicate visually that something is happening in the bkgd */
    buttons.nextStep.disabled = true
    downloadConfigTracker = false
    click.openLoader()

    setTimeout(() => {
      try {
        inlineVariables.formErrorMessage.innerHTML = ''
        // generate device bundle
        // derive a device root seed from the master
        const deviceRoot = master.derive(deviceNumber, {
          bundleType: 'deviceRoot'
        })
        deviceRoot.setAppData({
          device_number: deviceNumber,
          generate_by: "quickstart-v2.0"
        })
        // encrypts it with password: pass
        let pubKey = deviceRoot.signPubKey
        const pw = (new TextEncoder()).encode('pass')
        const encodedBytes = deviceRoot.lock([
          new hcSeedBundle.SeedCipherPwHash(
            hcSeedBundle.parseSecret(pw), 'minimum')
        ])

        // DEV MODE - check pub key for devices:
        console.log("Created from master seed: ", master.signPubKey);
        console.log(`Device ${deviceNumber}: ${toBase64(encodedBytes)}`)
        console.log(`Device signPubkey: ${pubKey}`)

        // pass seed into the blob
        let seed = {
          derivationPath: deviceNumber,
          // base64 encode it URLSAFE_NO_PADDING
          deviceRoot: toBase64(encodedBytes),
          pubKey
        }
        // Generate hpos-config.json and create download blob attached to url
        generateBlob(user, seed)
        // clear our secrets
        deviceRoot.zero()
      } catch (e) {
        inlineVariables.formErrorMessage.innerHTML = errorMessages.generateConfig
        throw new Error(`Error executing generateBlob with an error.  Error: ${e}`)
      }
      /* Clean State */
      click.closeLoader()
      updateUiStep(5)
      updateProgressBar(4)

      /* Reset Password inputs */
      inputs.password.value = ''
      inputs.passwordCheck.value = ''
    }, 1500)
  }

  /**
   * Generate save link of hpos-config.json and attach to `button` domElement
   *
   * @param {Object} user
   * @param {Object} seed {derivationPath, deviceRoot, pubKey}
  */
  const generateBlob = (user, seed) => {
    const configData = config(user.email, user.password, user.registrationCode.trim(), seed.derivationPath.toString(), seed.deviceRoot, seed.pubKey)
    const configBlob = new Blob([configData.config], { type: 'application/json' })

    /* NB: Do not delete!  Keep the below in case we decide to use the HoloPort url it is available right here */
    // console.log('Optional HoloPort url : ', configData.url)
    deviceID = configData.id
    configFileBlob = configBlob

    return configFileBlob
  }

  /**
  * Verify config was saved before allowing progression to next page
  *
  * @param {Boolean} downloadSeedComplete
 */
  const verifySeedDownloadComplete = (downloadSeedComplete = downloadSeedTracker, newConfig = resetUserConfig) => {
    if (downloadSeedComplete) {
      buttons.nextStep.disabled = false
      buttons.genSeed.disabled = true
    }
    else if (newConfig) {
      buttons.genSeed.classList.remove('disabled')
      buttons.genSeed.innerHTML = 'Generate & Save Master Seed*'
      buttons.nextStep.disabled = true
      resetUserConfig = false
    }
    else return buttons.nextStep.disabled = true
  }

  /**
   * Verify config was saved before allowing progression to next page
   *
   * @param {Boolean} downloadConfigComplete
  */
  const verifyDownloadComplete = (downloadConfigComplete = downloadConfigTracker, newConfig = resetUserConfig) => {
    if (downloadConfigComplete) {
      buttons.nextStep.disabled = false
    }
    else if (!downloadConfigComplete && newConfig) {
      buttons.nextStep.disabled = true
      resetUserConfig = false
      buttons.download.innerHTML = 'Save New Configuration File'
    }
    else return buttons.nextStep.disabled = true
  }

  /**
   * Reset Form Input Feilds while form is active
   *
   * @param {Array} inputElements
  */
  const resetFields = (inputElements) => {
    for (let inputElement of inputElements) {
      document.querySelector(`#${inputElement.id}-form-item`).classList.remove('error-red')
      try {
        inputElement.parentElement.querySelector('.input-item-label').classList.remove('error-red')
      } catch (e) {/* label does not exist */ }
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
      try {
        errorField.parentElement.querySelector('.input-item-label').classList.add('error-red')
      } catch (e) {/* label does not exist */ }
      if (errorMessage === errorMessages.missingFields) inlineVariables.formErrorMessage.innerHTML = errorMessage
      else document.querySelector(`#${errorField.id}-error-message`).innerHTML = errorMessage
    }
    return errorMessage
  }

  /**
   * Verify all form input before allowing progression to next page
  */
  const verifyInputData = () => {
    let inputValidity = false;
    if (stepTracker == 1) {
      inputValidity = confirmValidCode()
      if (inputValidity) buttons.nextStep.disabled = false
      else buttons.nextStep.disabled = true
    } if (stepTracker == 2) {
      inputValidity = confirmValidPassPhrase()
      if (inputValidity) {
        buttons.genSeed.disabled = false
      } else {
        buttons.genSeed.disabled = true
      }
    } else if (stepTracker == 4) {
      inputValidity = confirmValidInput()
      if (inputValidity) {
        buttons.nextStep.disabled = false
      } else {
        buttons.nextStep.disabled = true
      }
    }
    return inputValidity
  }

  /**
   * Input form error check
   *
  */
  const confirmValidInput = (submitPressed = signalKeyGen) => {
    const inputElements = Object.values(inputs)
    resetFields(inputElements)
    if (submitPressed) {
      if (!inputs.email.value) {
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
  const confirmValidCode = () => {
    const inputElements = Object.values({ registrationCode: inputs.registrationCode })
    resetFields(inputElements)
    if (!inputs.registrationCode.value) {
      const missingFields = inputElements.filter(inputs => !inputs.value)
      renderInputError(errorMessages.missingFields, missingFields)
    } else if (!validateRegistrationCode(inputs.registrationCode.value)) {
      renderInputError(errorMessages.registrationCode, [inputs.registrationCode])
    } else return true
  }
  const confirmValidPassPhrase = () => {
    const inputElements = Object.values({ seedPassphrase: inputs.seedPassphrase })
    resetFields(inputElements)
    if (!inputs.seedPassphrase.value) {
      const missingFields = inputElements.filter(inputs => !inputs.value)
      renderInputError(errorMessages.missingFields, missingFields)
    } else if (!validatePassphrae(inputs.seedPassphrase.value)) {
      renderInputError(errorMessages.seedPassphrase, [inputs.seedPassphrase])
    } else return true
  }

})()
