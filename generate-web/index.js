import './style.css'

(async () => {
  const { config } = await import('./pkg')

  const DOWNLOAD_FILE_NAME = 'holo-config.json'

  // Parse UI elements
  const buttons = {
    start: document.querySelector('#startButton'),
    generate: document.querySelector('#generateButton'),
    download: document.querySelector('#downloadButton'),
    copied: document.querySelector('#copiedButton'),
    openOlay: document.querySelector('#open-overlay'),
    closeOlay: document.querySelector('#close-overlay')
  }

  const inputs = {
    email: document.querySelector('#email'),
    password: document.querySelector('#password')
  }

  const user = {
    email: '',
    password: ''
  }

  // Actions executed at button click
  const click = {
    start: () => {
      if (!validateBrowser()) {
        alert('Please upgrade your browser to newer version.')
        return null
      }

      updateUiStep(1)
      updateProgressBar(1)
    },
    generate: () => {
      // Read inputs
      user.email = inputs.email.value
      user.password = inputs.password.value

      // Check for email and pass
      if (!validateEmail(user.email)) {
        alert('Wrong format of email')
        return
      } else if (!user.password) {
        alert('Password cannot be empty')
        return null
      }

      // Communicate visually that something is happening in the bkgd
      buttons.generate.disabled = true
      buttons.generate.innerText = 'Generating...'

      // Move generateDownload out of exec flow
      setTimeout(() => {
        // Generate holo-config.json and create download blob attached to url
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
        updateProgressBar(2)
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
        updateProgressBar(3)
      }, 1000)
    },
    copied: () => {
      updateUiStep(4)
      updateProgressBar(4)
    },
    openOlay: () => {
      document.querySelector('#fixed-overlay').style.display = 'block'
    },
    closeOlay: () => {
      document.querySelector('#fixed-overlay').style.display = 'none'
    }
  }

  // Bind actions to buttons
  buttons.start.onclick = click.start
  buttons.generate.onclick = click.generate
  buttons.download.onclick = click.download
  buttons.copied.onclick = click.copied
  buttons.openOlay.onclick = click.openOlay
  buttons.closeOlay.onclick = click.closeOlay

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
    document.body.className = 'step' + step
  }

  /**
   * Update the progresss bar
   * @param {int} currentStep
   * @param {bool} rewind
   */
  const updateProgressBar = (currentStep, rewind=false) => {
    if (!validation[currentStep]) {
      console.log(`Wrong parameter ${currentStep} in updateProgressBar()`)
      return null
    }

    const progressbarElement = document.getElementById('progressbar')
    console.log('progressbarElement : ', progressbarElement)

    if (rewind) {
      const previousStep = currentStep + 1
      console.log('previousStep : ', previousStep)
      // activate previous currentStep on progressbar 
      progressbarElement.childNodes[currentStep].removeClass('active')
      progressbarElement.childNodes[nextStep].addClass('active')
    }
    else {
      const nextStep = currentStep + 1
      console.log('nextStep : ', nextStep)
      // activate next currentStep on progressbar 
      progressbarElement.childNodes[nextStep].addClass('active')
    }
  }


    /**
   * Generate download link of holo-config.json and attach to `button` domElement
   *
   * @param {Object} user
   * @param {DomElement} button - a DomElement that will have download and attribute props updated
   */
  const generateDownload = (user, button) => {
    console.log('generating...')
    const configData = config(user.email, user.password)
    const configBlob = new Blob([configData.config], { type: 'application/json' })
    const url = URL.createObjectURL(configBlob)

    if (button.nodeName !== 'A') throw new Error('Download button has to be node <a> type')

    button.href = url
    button.download = DOWNLOAD_FILE_NAME

    // In case we decide to use the HoloPort url it is available right here
    console.log(configData.url)
  }
})()
