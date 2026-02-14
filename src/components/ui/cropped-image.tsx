import { useEffect, useState } from 'react'
import { tauriInvoke } from '@/lib/tauri/invoke'
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
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    const cached = cropCache.get(key)
    if (cached) {
      setSrc(cached)
      setError(null)
      return
    }

    setSrc(undefined)
    setError(null)

    let request = inflightCropRequests.get(key)
    if (!request) {
      request = tauriInvoke<string | null>('disciplines_cropped_image_get', { disciplineName, crop, label }).finally(
        () => {
          inflightCropRequests.delete(key)
        }
      )
      inflightCropRequests.set(key, request)
    }

    let alive = true

    request
      .then(b64 => {
        if (!alive) return
        if (!b64) {
          setError('No image data returned')
          return
        }

        const dataUrl = `data:image/png;base64,${b64}`
        cropCache.set(key, dataUrl)
        setSrc(dataUrl)
      })
      .catch(err => {
        if (!alive) return
        const message = err instanceof Error ? err.message : String(err)
        console.error('[cropped-image] failed to fetch crop', { disciplineName, label, crop, message })
        setError(message)
      })

    return () => {
      alive = false
    }
  }, [key, crop, disciplineName, label])

  if (!src) {
    if (error) {
      return (
        <div
          className={`border border-destructive/30 bg-destructive/5 ${className ?? ''}`}
          style={style}
          title={error}
        />
      )
    }

    return <div className={className} style={style} />
  }

  return <img src={src} alt="" className={`object-cover ${className ?? ''}`} style={style} />
}
