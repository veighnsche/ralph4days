import { useState } from 'react'

export type Page = 'tasks' | 'features' | 'disciplines'

export function useNavigation() {
  const [currentPage, setCurrentPage] = useState<Page>('tasks')

  return {
    currentPage,
    setCurrentPage
  }
}
