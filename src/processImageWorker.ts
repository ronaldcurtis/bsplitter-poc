import init, { greet, process_video_frame } from '../bsplitter-wasm/pkg'

type InitMessage = {
  type: 'init',
  targetCanvas: HTMLCanvasElement
  videoReadableStream: ReadableStream<VideoFrame>
}

type Message = InitMessage
let canvas: HTMLCanvasElement
let ctx: CanvasRenderingContext2D

self.onmessage = async (e: MessageEvent<Message>) => {
  const {type} = e.data
  if (type === 'init') {
    const { videoReadableStream } = e.data
    await init()
    greet()
    canvas = e.data.targetCanvas
    const context = canvas.getContext('2d')
    if (context) ctx = context
    const reader = videoReadableStream.getReader()
    // eslint-disable-next-line no-constant-condition
    while (true) {
      const result = await reader.read()
      if (result.done) break;
      const frame = result.value;
      const rawFrameData = new Uint8Array(3 * frame.codedWidth * frame.codedHeight / 2)
      await frame.copyTo(rawFrameData)
      process_video_frame(ctx, rawFrameData, frame.codedWidth, frame.codedHeight)
      frame.close()
    }
  }
}
