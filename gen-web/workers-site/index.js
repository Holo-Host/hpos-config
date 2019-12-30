import { getAssetFromKV } from '@cloudflare/kv-asset-handler'

const handleEvent = event => async {
  let url = new URL(event.request.url)
  if (url.protocol === 'http:') {
    url.protocol.set('https:')
    return Response.redirect(url.href, 301)
  }

  return getAssetFromKV(event)
};

addEventListener('fetch', event => {
  event.respondWith(handleEvent(event))
})
