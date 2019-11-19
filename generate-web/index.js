import './style.css'

(async () => {
  const { config } = await import('./pkg')

  const DOWNLOAD_FILE_NAME = 'hpos-state.json'

  // Parse UI elements
  const buttons = {
    start: document.querySelector('#startButton'),
    generate: document.querySelector('#generateButton'),
    download: document.querySelector('#downloadButton'),
    copied: document.querySelector('#copiedButton'),
    openOlay: document.querySelector('#open-overlay'),
    closeOlay: document.querySelector('#close-overlay'),
    back2: document.querySelector('#backButton2'),
    back3: document.querySelector('#backButton3')
  }

  const inputs = {
    email: document.querySelector('#email'),
    password: document.querySelector('#password'),
    passwordCheck: document.querySelector('#password-check'),
    deviceName: document.querySelector('#device-name')
  }

  const inlineVariables = {
    emailPlaceholder: document.querySelector('#emailPlaceholder')
  } 

  const user = {
    email: '',
    password: '',
    device_name: ''
  }

  // Actions executed at button click
  const click = {
    start: () => {
      if (!validateBrowser()) {
        alert('Please upgrade your browser to newer version.')
        return null
      }

      updateUiStep(1)
    },
    generate: () => {
      // Read inputs
      user.email = inputs.email.value
      user.password = inputs.password.value
      user.device_name = inputs.deviceName.value
      
      
      console.log('user config : ', user)
      

      // Check for email and pass
      if(!user.password && !user.email && !user.device_name) {
        alert('Form cannot be empty')
        return null
      }
      else if (!validateEmail(user.email)) {
        alert('Wrong format of email')
        return null
      } else if (!user.password) {
        alert('Password cannot be empty')
        return null
      // Verify password and passwordCheck match
      } else if (inputs.password.value !== inputs.passwordCheck.value) {
        alert('Passwords do not match')
        return null
      } else if (!user.device_name) {
        alert('Device cannot be empty')
        return null
      }

      // Communicate visually that something is happening in the bkgd
      buttons.generate.disabled = true
      buttons.generate.innerText = 'Generating...'

      // Move generateDownload out of exec flow
      setTimeout(() => {
        // Generate hpos-state.json and create download blob attached to url
        try {
          generateDownload(user, buttons.download)
        } catch (e) {
          console.log(`Error executing generateDownload with an error ${e}`)
          return null
        }

        // revert UI
        buttons.generate.disabled = false
        buttons.generate.innerText = 'Generate'
        updateUiStep(2)
        updateProgressBar(1)
      }, 50)
    },
    download: () => {
      // Communicate visually that something is happening in the bkgd
      buttons.download.disabled = true
      buttons.download.innerText = 'Downloading...'

      // Update user email in the UI
      document.querySelector('#emailPlaceholder').innerText = user.email

      // revert
      setTimeout(() => {
        buttons.download.disabled = false
        buttons.download.innerText = 'Download'
        updateUiStep(3)
        updateProgressBar(2)
      }, 1000)
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
    back2: () => {
      const rewind = true
      updateProgressBar(2, rewind)
      updateUiStep(1)
    },
    back3: () => {
      const rewind = true
      updateProgressBar(3, rewind)
      updateUiStep(2)
    }
  }

  // Bind actions to buttons
  buttons.start.onclick = click.start
  buttons.generate.onclick = click.generate
  buttons.download.onclick = click.download
  buttons.copied.onclick = click.copied
  buttons.openOlay.onclick = click.openOlay
  buttons.closeOlay.onclick = click.closeOlay
  buttons.back2.onclick = click.back2
  buttons.back3.onclick = click.back3


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
    return document.body.className = 'step' + step
  }

  /**
   * Update the progresss bar
   * @param {int} currentTransition
   * @param {bool} rewind
   */
  const updateProgressBar = (currentTransition, rewind = false) => {
    console.log('rewind >> ', rewind)
    
    if (currentTransition <= 1) rewind = false
    if (!validation[currentTransition]) {
      console.log(`Wrong parameter ${currentTransition} in updateProgressBar()`)
      return null
    }
    const stepIndex = currentTransition - 1

    // Locate current step element and remove 'active' class
    const childListNodes = document.querySelectorAll('li.progressbar-item')
    const currentlyActive = childListNodes[stepIndex]
    console.log('currentlyActive : ', currentlyActive.classList)
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
})()
