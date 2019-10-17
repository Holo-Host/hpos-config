(() => {
  // Parse query string from URL
  const qs = ((a) => {
    if (a === '') return {};
    var b = {}
    for (var i = 0; i < a.length; ++i) {
      var p = a[i].split('=')
      if (p.length !== 2) continue
      b[p[0]] = decodeURIComponent(p[1].replace(/\+/g, ' '))
    }
    return b
  })(window.location.search.substr(1).split('&'))

  /**
   * Keep fetching given url until successful 200 response in 2s intervals
   * @param {string} url 
   * @returns {Promise}
   */
  function fetchRetry(url) {
    const fetchOptions = {
      cache: 'no-cache'
    }
    const delay = 2000

    return new Promise((resolve,reject) => {
        const success = (r) => {
          if (r.status === 200) {
            console.log('Fetch success!')
            resolve(r)
          } else {
            console.error('Error in response status: ', r.status, ' retrying')
            setTimeout(fetchUrl,delay)
          }
        }
        const failure = (e) => {
          console.error('Fetch error:', e)
          setTimeout(fetchUrl,delay)
        }
        const fetchUrl = () => {
            return fetch(url + 'ping/',fetchOptions)
                .then(success)
                .catch(failure)
        }
        fetchUrl();
    });
  }

  // Check if we have received HP's url in the query string
  if (!qs.url) {
    console.log('No HoloPort URL found in the query')
    return
  }

  // Clean up url
  const url = 'http://' + qs.url.replace(/^https?:\/\//,'').replace(/\/$/, "") + '/'

  // Start polling HP to check if it's up and running yet
  fetchRetry(url)
    .then(r => {
      // On success print HP's url for user
      document.querySelector('#hp-status-registering').innerHTML = `Registered! Click <a href="${url}">${url}</a> to access your HoloPort Admin Panel.`
    })

})()


 