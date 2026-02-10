import { invoke } from '@tauri-apps/api/core'
import { useEffect, useState } from 'react'
import type { CropBoxData } from '@/types/generated'

const cropCache = new Map<string, string>()

function cacheKey(name: string, label: string, crop: CropBoxData): string {
  return `${name}|${label}|${crop.x},${crop.y},${crop.w},${crop.h}`
}

export function CroppedImage({
  disciplineName,
  label,
  crop,
  className,
  style
}: {
  disciplineName: string
  label: string
  crop: CropBoxData
  className?: string
  style?: React.CSSProperties
}) {
  const key = cacheKey(disciplineName, label, crop)
  const [src, setSrc] = useState<string | undefined>(cropCache.get(key))

  useEffect(() => {
    if (cropCache.has(key)) {
      setSrc(cropCache.get(key))
      return
    }

    invoke<string | null>('get_cropped_image', { name: disciplineName, crop, label }).then(b64 => {
      if (b64) {
        const dataUrl = `data:image/png;base64,${b64}`
        cropCache.set(key, dataUrl)
        setSrc(dataUrl)
      }
    })
  }, [key, disciplineName, crop, label])

  if (!src) return <div className={className} style={style} />

  return <img src={src} alt="" className={`object-cover ${className ?? ''}`} style={style} />
}
