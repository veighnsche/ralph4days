import { invoke } from '@tauri-apps/api/core'
import { useEffect, useState } from 'react'
import type { DisciplineCropsData } from '@/types/generated'

interface DisciplineImageEntry {
  imageUrl: string
  crops?: DisciplineCropsData
}

export function useDisciplineImageStore(
  disciplines: { name: string; imagePath?: string; crops?: DisciplineCropsData }[]
) {
  const [store, setStore] = useState<Map<string, DisciplineImageEntry>>(new Map())

  useEffect(() => {
    const withImages = disciplines.filter(d => d.imagePath)
    if (withImages.length === 0) return

    let cancelled = false
    Promise.all(
      withImages.map(async d => {
        const b64 = await invoke<string | null>('get_discipline_image_data', { name: d.name })
        return { name: d.name, b64, crops: d.crops } as const
      })
    ).then(results => {
      if (cancelled) return
      const map = new Map<string, DisciplineImageEntry>()
      for (const { name, b64, crops } of results) {
        if (b64) map.set(name, { imageUrl: `data:image/png;base64,${b64}`, crops })
      }
      setStore(map)
    })

    return () => {
      cancelled = true
    }
  }, [disciplines])

  return store
}
