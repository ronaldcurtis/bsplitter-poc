/* eslint-disable no-restricted-globals */

type InitMessage = {
  type: 'init',
  targetCanvas: HTMLCanvasElement
}

type ProcessImageMessage = {
  type: 'process_image',
  image: Uint8ClampedArray
  height: number
  width: number
}

type Message = InitMessage | ProcessImageMessage
let canvas: HTMLCanvasElement

self.onmessage = (e: MessageEvent<Message>) => {
  const {type} = e.data
  if (type === 'init') {
    canvas = e.data.targetCanvas
    canvas.getContext('2d')
  } else if (type === 'process_image') {
    const { image, width, height } = e.data
    if (canvas) {
      canvas.getContext('2d')?.putImageData(new ImageData(image, width, height), 0, 0)
    }
  }
}
