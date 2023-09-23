import './App.css'
import {useEffect, useRef} from "react";
import ProcessImageWorker from './processImageWorker.ts?worker'

const worker = new ProcessImageWorker()

function App() {
  const targetCanvasRef = useRef<HTMLCanvasElement | null>(null)
  useEffect(() => {
    async function init() {
      const camStream = await navigator.mediaDevices.getUserMedia({video: true})
      const track = camStream.getVideoTracks()[0]
      const trackProcessor = new MediaStreamTrackProcessor({ track })
      const readableStream = trackProcessor.readable;
      const videoSettings = track.getSettings()
      if (targetCanvasRef.current) {
        targetCanvasRef.current.width = videoSettings.width ?? 0
        targetCanvasRef.current.height = videoSettings.height ?? 0
        const offscreenCanvas = targetCanvasRef.current.transferControlToOffscreen()
        worker.postMessage({ type: 'init', targetCanvas: offscreenCanvas, videoReadableStream: readableStream }, [offscreenCanvas, readableStream])
      }
    }
    init().then(() => undefined)
  }, [])

  return (
    <>
      <h1>Rust Wasm Video Processing POC</h1>
      <div className="canvasCon">
        <canvas ref={targetCanvasRef} id="targetCanvas" />
      </div>
    </>
  )
}

export default App
