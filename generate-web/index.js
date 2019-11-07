import { saveAs } from 'file-saver'

(async () => {
  const { state } = await import('./pkg')

  const elements = {
    email: document.querySelector('#email'),
    form: document.querySelector('#form'),
    generate: document.querySelector('#generate'),
    password: document.querySelector('#password')
  }

  elements.form.addEventListener('keydown', e => {
    if (event.key === 'Enter') { elements.generate.click() }
  })

  elements.generate.addEventListener('click', e => {
    const stateData = state(elements.email.value, elements.password.value)
    const stateBlob = new Blob([stateData.state], { type: 'application/json' })

    saveAs(stateBlob, 'hpos-state.json')
    alert(stateData.url)
  })
})()
