import { Bot, User } from 'lucide-react'
import { Avatar, AvatarFallback } from '@/components/ui/avatar'
import { CroppedImage } from '@/components/ui/cropped-image'
import { useDisciplines } from '@/hooks/disciplines'

export function CommentAvatar({ discipline }: { discipline?: string | null }) {
  const { disciplines } = useDisciplines()

  if (discipline) {
    const disc = disciplines.find(d => d.name === discipline)
    if (disc?.crops?.face) {
      return (
        <div className="size-12 flex-shrink-0 rounded-md overflow-hidden self-start">
          <CroppedImage
            disciplineName={disc.name}
            label="comment-face"
            crop={disc.crops.face}
            className="size-full object-cover"
          />
        </div>
      )
    }
  }

  return (
    <Avatar className="size-12 flex-shrink-0 rounded-md">
      <AvatarFallback className="text-muted-foreground">
        {discipline ? <Bot className="h-5 w-5" /> : <User className="h-5 w-5" />}
      </AvatarFallback>
    </Avatar>
  )
}
