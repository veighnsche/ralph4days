import { useEffect, useState } from 'react'
import type { CropBoxData } from '@/types/generated'

export function CroppedImage({
  src,
  crop,
  className,
  style
}: {
  src: string
  crop: CropBoxData
  className?: string
  style?: React.CSSProperties
}) {
  const [croppedSrc, setCroppedSrc] = useState<string>()

  useEffect(() => {
    const img = new Image()
    img.onload = () => {
      const canvas = document.createElement('canvas')
      const sx = Math.round(crop.x * img.naturalWidth)
      const sy = Math.round(crop.y * img.naturalHeight)
      const sw = Math.round(crop.w * img.naturalWidth)
      const sh = Math.round(crop.h * img.naturalHeight)
      canvas.width = sw
      canvas.height = sh
      const ctx = canvas.getContext('2d')
      if (!ctx) return
      ctx.drawImage(img, sx, sy, sw, sh, 0, 0, sw, sh)
      setCroppedSrc(canvas.toDataURL('image/png'))
    }
    img.src = src
  }, [src, crop.x, crop.y, crop.w, crop.h])

  if (!croppedSrc) return <div className={className} style={style} />

  return <img src={croppedSrc} alt="" className={`object-cover ${className ?? ''}`} style={style} />
}
