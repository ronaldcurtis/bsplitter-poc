import './App.css'
import {useEffect, useRef} from "react";

function renderVideoToCanvas(canvas: HTMLCanvasElement, video: HTMLVideoElement) {
  const ctx = canvas.getContext('2d')
  function render() {
    if (ctx) {
      ctx.drawImage(video, 0, 0, 400, 300)
      requestAnimationFrame(render)
    }
  }
  render()
}

function App() {
  const videoRef = useRef<HTMLVideoElement | null>(null)
  const mainCanvasRef = useRef<HTMLCanvasElement | null>(null)
  useEffect(() => {
    async function setVideo() {
      const camStream = await navigator.mediaDevices.getUserMedia({video: true})
      if (videoRef.current) {
        videoRef.current.srcObject = camStream
        if (mainCanvasRef.current) {
          renderVideoToCanvas(mainCanvasRef.current, videoRef.current)
        }
      }
    }
    setVideo().then(() => undefined)
  }, [])

  return (
    <>
      <h1>Webcam processing POC</h1>
      <video ref={videoRef} className="vid" autoPlay={true} />
      <canvas ref={mainCanvasRef} id="mainCanvas" width="400" height="300" />
    </>
  )
}

export default App
