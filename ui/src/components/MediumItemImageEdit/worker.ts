import { imageSize } from 'image-size'

self.addEventListener('message', (e: MessageEvent<ArrayBuffer>) => {
  self.postMessage(imageSize(new Uint8Array(e.data)))
})
