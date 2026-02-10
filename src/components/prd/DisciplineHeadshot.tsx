import { useMemo } from 'react'
import { CroppedImage } from '@/components/ui/cropped-image'
import type { CropBoxData } from '@/types/generated'

interface DisciplineHeadshotProps {
  disciplineName: string
  faceCrop: CropBoxData
}

export function DisciplineHeadshot({ disciplineName, faceCrop }: DisciplineHeadshotProps) {
  const wideCrop = useMemo<CropBoxData>(
    () => ({
      x: faceCrop.x,
      y: faceCrop.y,
      w: Math.min(faceCrop.w * 2, 1 - faceCrop.x),
      h: Math.min(1 - faceCrop.y, faceCrop.w * 2)
    }),
    [faceCrop.x, faceCrop.y, faceCrop.w]
  )

  return (
    <div className="absolute left-0 top-0 w-44 h-full pointer-events-none overflow-hidden">
      <CroppedImage
        disciplineName={disciplineName}
        label="headshot"
        crop={wideCrop}
        className="w-full"
        style={{
          maskImage: 'linear-gradient(to right, rgba(0,0,0,0.67) 0%, rgba(0,0,0,0.67) 33%, transparent 100%)',
          WebkitMaskImage: 'linear-gradient(to right, rgba(0,0,0,0.67) 0%, rgba(0,0,0,0.67) 33%, transparent 100%)'
        }}
      />
    </div>
  )
}
