import { saveAs } from 'file-saver';

async function main() {
  const { config } = await import('./pkg');

  const elements = {
    email: document.querySelector('#email'),
    form: document.querySelector('#form'),
    generate: document.querySelector('#generate'),
    password: document.querySelector('#password')
  };

  elements.form.addEventListener('keydown', e => {
    if (event.key === "Enter")
      elements.generate.click();
  });

  elements.generate.addEventListener('click', e => {
    const configData = config(elements.email.value, elements.password.value);
    const configBlob = new Blob([configData.config], {type: 'application/json'});

    saveAs(configBlob, 'holo-config.json');
    alert(configData.url);
  });
};

main();
