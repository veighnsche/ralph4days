import type React from 'react'
import { Card } from '@/components/ui/card'
import { ScrollArea } from '@/components/ui/scroll-area'

interface DetailPageLayoutProps {
  accentColor: string
  cardBorderColor?: string
  cardOverlay?: React.ReactNode
  sidebarImage?: React.ReactNode
  mainContent: React.ReactNode
  sidebar: React.ReactNode
  children?: React.ReactNode
}

export function DetailPageLayout({
  accentColor,
  cardBorderColor,
  cardOverlay,
  sidebarImage,
  mainContent,
  sidebar,
  children
}: DetailPageLayoutProps) {
  return (
    <div
      className="h-full px-3 relative"
      style={{
        background: `repeating-linear-gradient(
        45deg,
        transparent,
        transparent 10px,
        ${accentColor}15 10px,
        ${accentColor}15 20px
      )`
      }}>
      <ScrollArea className="h-full">
        <div className="py-3 space-y-3">
          <Card
            className="shadow-sm flex flex-row gap-0 py-0 relative"
            style={cardBorderColor ? { borderColor: cardBorderColor } : undefined}>
            <div className="flex-1 min-w-0 py-4 relative z-10">
              {cardOverlay && (
                <div className="absolute inset-0 overflow-hidden pointer-events-none z-20">{cardOverlay}</div>
              )}
              {mainContent}
            </div>

            <div
              className="w-56 flex-shrink-0 border-l relative overflow-hidden z-10"
              style={cardBorderColor ? { borderColor: cardBorderColor } : undefined}>
              {sidebarImage}
              <div className="relative">{sidebar}</div>
            </div>
          </Card>

          {children}
        </div>
      </ScrollArea>
    </div>
  )
}
