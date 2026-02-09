import { useMemo } from 'react'
import { CroppedImage } from '@/components/ui/cropped-image'
import type { CropBoxData } from '@/types/generated'

interface DisciplineHeadshotProps {
  imageUrl: string
  faceCrop: CropBoxData
  bgColor: string
}

export function DisciplineHeadshot({ imageUrl, faceCrop, bgColor: _bgColor }: DisciplineHeadshotProps) {
  const wideCrop = useMemo<CropBoxData>(
    () => ({ x: faceCrop.x, y: faceCrop.y, w: Math.min(faceCrop.w * 2, 1 - faceCrop.x), h: faceCrop.h }),
    [faceCrop.x, faceCrop.y, faceCrop.w, faceCrop.h]
  )

  return (
    <div className="absolute left-0 top-0 w-44 h-22 pointer-events-none overflow-hidden">
      <CroppedImage
        src={imageUrl}
        crop={wideCrop}
        className="absolute inset-0 h-full w-full"
        style={{
          maskImage: 'linear-gradient(to right, rgba(0,0,0,0.67) 0%, rgba(0,0,0,0.67) 33%, transparent 100%)',
          WebkitMaskImage: 'linear-gradient(to right, rgba(0,0,0,0.67) 0%, rgba(0,0,0,0.67) 33%, transparent 100%)'
        }}
      />
    </div>
  )
}
