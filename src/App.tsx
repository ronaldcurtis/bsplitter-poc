import './App.css'
import {useEffect, useRef} from "react";
import ProcessImageWorker from './processImageWorker.ts?worker'


const worker = new ProcessImageWorker()

function renderAndProcessVideo(canvas: HTMLCanvasElement, video: HTMLVideoElement) {
  const ctx = canvas.getContext('2d')
  function render() {
    if (ctx) {
      ctx.drawImage(video, 0, 0, 400, 300)
      const image = ctx.getImageData(0, 0, 400, 300)
      worker.postMessage({type: 'process_image', image: image.data, width: 400, height: 300 }, [image.data.buffer])
      requestAnimationFrame(render)
    }
  }
  render()
}

function App() {
  const videoRef = useRef<HTMLVideoElement | null>(null)
  const targetCanvasRef = useRef<HTMLCanvasElement | null>(null)
  const sourceCanvasRef = useRef<HTMLCanvasElement | null>(null)
  useEffect(() => {
    async function setVideo() {
      const camStream = await navigator.mediaDevices.getUserMedia({video: true})
      if (videoRef.current) {
        videoRef.current.srcObject = camStream
        if (sourceCanvasRef.current && targetCanvasRef.current) {
          const offscreenCanvas = targetCanvasRef.current.transferControlToOffscreen()
          worker.postMessage({ type: 'init', targetCanvas: offscreenCanvas }, [offscreenCanvas])
          renderAndProcessVideo(sourceCanvasRef.current, videoRef.current)
        }
      }
    }
    setVideo().then(() => undefined)
  }, [])

  return (
    <>
      <h1>Webcam processing POC</h1>
      <video ref={videoRef} className="vid" autoPlay={true} />
      <div className="canvasCon">
        <canvas ref={targetCanvasRef} id="targetCanvas" width="400" height="300" />
        <canvas ref={sourceCanvasRef} id="sourceCanvas" width="400" height="300" />
      </div>
    </>
  )
}

export default App
