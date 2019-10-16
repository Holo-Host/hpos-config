import { saveAs } from 'file-saver'

async () => {

  elements.generate.addEventListener('click', e => {
    
    const configBlob = new Blob([configData.config], { type: 'application/json' })

    saveAs(configBlob, 'holo-config.json')
    alert(configData.url)
  })
}

(async () => {
  const { config } = await import('./pkg')

  const DOWNLOAD_FILE_NAME = 'holo-config.json';

  // Parse UI elements
  const buttons = {
    start: document.querySelector('#startButton'),
    generate: document.querySelector('#generateButton'),
    download: document.querySelector('#downloadButton'),
    copied: document.querySelector('#copiedButton')
  }

  const inputs = {
    email: document.querySelector('#email'),
    password: document.querySelector('#password')
  }

  const user ={
    email: "",
    password: ""
  }

  const clickStart = () => {
    updateUiStep(1);
  }

  const clickGenerate = () => {
    // Read inputs
    user.email = inputs.email.value;
    user.password = inputs.password.value;

    // Check for email and pass
    if (!validateEmail(user.email)) {
        alert("Wrong format of email");
        return;
    } else if (!user.password) {
        alert("Password cannot be empty");
        return;
    }

    // Communicate visually that something is happening in the bkgd
    buttons.generate.disabled = true;
    buttons.generate.innerText = "Generating...";

    // Generate holo-config.json and create download blob attached to url
    try {
      generateDownload(user, buttons.download);
    } catch(e) {
      console.log(`Error executing generateDownload with an error ${e}`);
      return;
    }

    // revert
    setTimeout(() => {
        buttons.generate.disabled = false;
        buttons.generate.innerText = "Generate";
        updateUiStep(2);
    }, 1000);
  }

  const clickDownload = () => {
    // Communicate visually that something is happening in the bkgd
    buttons.download.disabled = true;
    buttons.download.innerText = "Downloading...";

    // Update user email in the UI
    document.querySelector("#emailPlaceholder").innerText = user.email;

    // revert
    setTimeout(() => {
        buttons.download.disabled = false;
        buttons.download.innerText = "Download";
        updateUiStep(3);
    }, 1000);
  }

  const clickCopied = () => {
    updateUiStep(4);
  }

  // Bind actions to buttons
  buttons.start.onclick = clickStart;
  buttons.generate.onclick = clickGenerate;
  buttons.download.onclick = clickDownload;
  buttons.copied.onclick = clickCopied;

  const validateEmail = (email) => {
    let re = /^\S+@\S+$/;
    return re.test(String(email).toLowerCase());
  }

  const updateUiStep = (step) => {
    let validation = {0:!0, 1:!0, 2:!0, 3:!0, 4:!0};

    if (!validation[step]) {
        console.log(`Wrong parameter ${step} in updateUiStep()`);
        return;
    }

    document.body.className = 'step' + step;
  }

  /**
   * 
   * @param {Object} user 
   * @param {DomElement} button - a DomElement that will have download and attribute props updated
   */

  const generateDownload = (user, button) => {
    const configData = config(user.email, user.password);
    const configBlob = new Blob([configData.config], { type: 'application/json' });
    const url = URL.createObjectURL(configBlob);

    if (button.nodeName !== "A") throw new Error("Download button has to be node <a> type");

    button.href = url;
    button.download = DOWNLOAD_FILE_NAME;

    // In case we decide to use the HoloPort url it is available right here
    console.log(configData.url);
  }
})()