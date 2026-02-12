import { invoke } from '@tauri-apps/api/core'
import { useEffect, useState } from 'react'
import type { CropBoxData } from '@/types/generated'

const cropCache = new Map<string, string>()
const inflightCropRequests = new Map<string, Promise<string | null>>()

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
    const cached = cropCache.get(key)
    if (cached) {
      setSrc(cached)
      return
    }

    let request = inflightCropRequests.get(key)
    if (!request) {
      request = invoke<string | null>('get_cropped_image', { name: disciplineName, crop, label }).finally(() => {
        inflightCropRequests.delete(key)
      })
      inflightCropRequests.set(key, request)
    }

    request.then(b64 => {
      if (b64) {
        const dataUrl = `data:image/png;base64,${b64}`
        cropCache.set(key, dataUrl)
        setSrc(dataUrl)
      }
    })
  }, [key])

  if (!src) return <div className={className} style={style} />

  return <img src={src} alt="" className={`object-cover ${className ?? ''}`} style={style} />
}
