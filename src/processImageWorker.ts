import init, { greet, process_image } from '../bsplitter-wasm/pkg'

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
let ctx: CanvasRenderingContext2D

self.onmessage = async (e: MessageEvent<Message>) => {
  const {type} = e.data
  if (type === 'init') {
    await init()
    greet()
    canvas = e.data.targetCanvas
    const context = canvas.getContext('2d')
    if (context) ctx = context
  } else if (type === 'process_image') {
    const { image, width, height } = e.data
    process_image(ctx, new Uint8Array(image), width, height)
  }
}
