import { useEffect, useState } from 'react'
import { api } from '@/api/client'
import type { FeatureDetail, Classification } from '@/lib/types'

export function useFeature(slug: string) {
  const [feature, setFeature] = useState<FeatureDetail | null>(null)
  const [classification, setClassification] = useState<Classification | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [loading, setLoading] = useState(true)

  const refresh = async () => {
    try {
      setLoading(true)
      const [f, c] = await Promise.all([
        api.getFeature(slug),
        api.getFeatureNext(slug),
      ])
      setFeature(f)
      setClassification(c)
      setError(null)
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to load feature')
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => { refresh() }, [slug])

  return { feature, classification, error, loading, refresh }
}
