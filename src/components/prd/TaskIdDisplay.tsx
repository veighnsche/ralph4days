import { Badge } from '@/components/ui/badge'
import { resolveIcon } from '@/lib/iconRegistry'
import type { CropBoxData, Task } from '@/types/generated'

interface TaskIdDisplayProps {
  task: Task
  variant?: 'default' | 'badge' | 'full'
  className?: string
  imageUrl?: string
  faceCrop?: CropBoxData
}

function formatTaskId(id: number): string {
  if (id > 999) {
    return id.toString()
  }
  return `#${id.toString().padStart(3, '0')}`
}

function ImageOrIcon({
  imageUrl,
  faceCrop,
  Icon,
  bgColor,
  borderColor
}: {
  imageUrl?: string
  faceCrop?: CropBoxData
  Icon: ReturnType<typeof resolveIcon>
  bgColor: string
  borderColor: string
}) {
  if (imageUrl) {
    return (
      <img
        src={imageUrl}
        alt=""
        className="w-12 h-12 rounded border object-cover"
        style={{
          borderColor,
          objectPosition: faceCrop
            ? `${(faceCrop.x + faceCrop.w / 2) * 100}% ${(faceCrop.y + faceCrop.h / 2) * 100}%`
            : '50% 15%'
        }}
      />
    )
  }

  return (
    <div
      className="w-12 h-12 flex items-center justify-center rounded border"
      style={{ backgroundColor: bgColor, borderColor }}>
      <Icon className="w-6 h-6" style={{ color: borderColor }} />
    </div>
  )
}

export function TaskIdDisplay({ task, variant = 'default', className = '', imageUrl, faceCrop }: TaskIdDisplayProps) {
  const DisciplineIcon = resolveIcon(task.disciplineIcon)
  const bgColor = `color-mix(in oklch, ${task.disciplineColor} 15%, transparent)`
  const formattedId = formatTaskId(task.id)

  if (variant === 'full') {
    return (
      <div className={`flex items-center gap-1.5 text-sm text-muted-foreground ${className}`}>
        <span>{task.featureDisplayName}</span>
        <span className="opacity-40">/</span>
        <span className="inline-flex items-center gap-1" style={{ color: task.disciplineColor }}>
          <DisciplineIcon className="w-3.5 h-3.5" />
          {task.disciplineDisplayName}
        </span>
        <span className="opacity-40">/</span>
        <span className="font-mono">{formattedId}</span>
      </div>
    )
  }

  if (variant === 'badge') {
    return (
      <div className={`flex items-center gap-2 ${className}`}>
        <ImageOrIcon
          imageUrl={imageUrl}
          faceCrop={faceCrop}
          Icon={DisciplineIcon}
          bgColor={bgColor}
          borderColor={task.disciplineColor}
        />

        <div className="flex flex-col items-start leading-tight">
          <Badge variant="outline" className="font-mono text-xs mb-0.5">
            {task.featureAcronym}
          </Badge>
          <Badge
            variant="outline"
            className="font-mono text-xs mb-0.5"
            style={{
              borderColor: task.disciplineColor,
              backgroundColor: bgColor,
              color: task.disciplineColor
            }}>
            {task.disciplineAcronym}
          </Badge>
          <Badge variant="outline" className="font-mono text-xs mb-0.5">
            {formattedId}
          </Badge>
        </div>
      </div>
    )
  }

  return (
    <div className={`flex items-center gap-2 ${className}`}>
      <ImageOrIcon
        imageUrl={imageUrl}
        faceCrop={faceCrop}
        Icon={DisciplineIcon}
        bgColor={bgColor}
        borderColor={task.disciplineColor}
      />

      <div className="flex flex-col items-start leading-tight font-mono">
        <span className="text-xs text-muted-foreground">{task.featureAcronym}</span>
        <span className="text-xs font-medium" style={{ color: task.disciplineColor }}>
          {task.disciplineAcronym}
        </span>
        <span className="text-xs text-muted-foreground">{formattedId}</span>
      </div>
    </div>
  )
}
