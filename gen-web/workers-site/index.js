import { getAssetFromKV } from '@cloudflare/kv-asset-handler'

const handleEvent = async event => {
  const url = new URL(event.request.url)

  if (url.protocol === 'http:') {
    url.protocol = 'https:'
    return Response.redirect(url.href, 301)
  }

  return getAssetFromKV(event)
}

addEventListener('fetch', event => {
  event.respondWith(handleEvent(event))
})
