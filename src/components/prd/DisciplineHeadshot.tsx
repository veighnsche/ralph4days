import type { CropBoxData } from '@/types/generated'

interface DisciplineHeadshotProps {
  imageUrl: string
  faceCrop: CropBoxData
  bgColor: string
}

export function DisciplineHeadshot({ imageUrl, faceCrop: _faceCrop, bgColor: _bgColor }: DisciplineHeadshotProps) {
  return (
    <div className="absolute left-0 top-0 w-44 h-22 pointer-events-none overflow-hidden">
      <img
        src={imageUrl}
        alt=""
        className="absolute inset-0 h-full w-full object-cover"
        style={{
          maskImage: 'linear-gradient(to right, rgba(0,0,0,0.67) 0%, rgba(0,0,0,0.67) 33%, transparent 100%)',
          WebkitMaskImage: 'linear-gradient(to right, rgba(0,0,0,0.67) 0%, rgba(0,0,0,0.67) 33%, transparent 100%)'
        }}
      />
    </div>
  )
}
