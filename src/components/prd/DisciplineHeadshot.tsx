import { useEffect, useMemo, useState } from 'react'
import type { CropBoxData } from '@/types/generated'

interface DisciplineHeadshotProps {
  imageUrl: string
  faceCrop: CropBoxData
  bgColor: string
}

export function DisciplineHeadshot({ imageUrl, faceCrop, bgColor: _bgColor }: DisciplineHeadshotProps) {
  const wideCrop = useMemo<CropBoxData>(
    () => ({
      x: faceCrop.x,
      y: faceCrop.y,
      w: Math.min(faceCrop.w * 2, 1 - faceCrop.x),
      h: Math.min(1 - faceCrop.y, faceCrop.w * 2)
    }),
    [faceCrop.x, faceCrop.y, faceCrop.w]
  )

  const [croppedSrc, setCroppedSrc] = useState<string>()

  useEffect(() => {
    const img = new Image()
    img.onload = () => {
      const canvas = document.createElement('canvas')
      const sx = Math.round(wideCrop.x * img.naturalWidth)
      const sy = Math.round(wideCrop.y * img.naturalHeight)
      const sw = Math.round(wideCrop.w * img.naturalWidth)
      const sh = Math.round(wideCrop.h * img.naturalHeight)
      canvas.width = sw
      canvas.height = sh
      const ctx = canvas.getContext('2d')
      if (!ctx) return
      ctx.drawImage(img, sx, sy, sw, sh, 0, 0, sw, sh)
      setCroppedSrc(canvas.toDataURL('image/png'))
    }
    img.src = imageUrl
  }, [imageUrl, wideCrop.x, wideCrop.y, wideCrop.w, wideCrop.h])

  return (
    <div className="absolute left-0 top-0 w-44 h-full pointer-events-none overflow-hidden">
      {croppedSrc && (
        <img
          src={croppedSrc}
          alt=""
          className="w-full"
          style={{
            maskImage: 'linear-gradient(to right, rgba(0,0,0,0.67) 0%, rgba(0,0,0,0.67) 33%, transparent 100%)',
            WebkitMaskImage: 'linear-gradient(to right, rgba(0,0,0,0.67) 0%, rgba(0,0,0,0.67) 33%, transparent 100%)'
          }}
        />
      )}
    </div>
  )
}
