import type { ReactNode } from 'react'

interface PageLayoutProps {
  children: ReactNode
}

export function PageLayout({ children }: PageLayoutProps) {
  return <div className="h-full flex flex-col overflow-hidden">{children}</div>
}

interface PageHeaderProps {
  children: ReactNode
}

export function PageHeader({ children }: PageHeaderProps) {
  return <div className="flex-shrink-0 p-3 pb-0">{children}</div>
}

interface PageContentProps {
  children: ReactNode
}

export function PageContent({ children }: PageContentProps) {
  return <div className="flex-1 min-h-0 overflow-auto p-3">{children}</div>
}

interface PageActionsProps {
  children: ReactNode
}

export function PageActions({ children }: PageActionsProps) {
  return <div className="flex-shrink-0 px-3 pb-2 flex gap-2">{children}</div>
}
