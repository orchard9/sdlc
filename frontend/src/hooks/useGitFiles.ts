import { useCallback, useEffect, useRef, useState } from 'react'

export interface GitFile {
  path: string
  status: string
  staged: boolean
  old_path?: string
}

interface GitFilesResponse {
  files: GitFile[]
}

export function useGitFiles(intervalMs = 10_000) {
  const [files, setFiles] = useState<GitFile[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState(false)
  const timerRef = useRef<ReturnType<typeof setInterval> | null>(null)

  const fetchFiles = useCallback(async () => {
    try {
      const res = await fetch('/api/git/files')
      if (!res.ok) throw new Error(res.statusText)
      const data: GitFilesResponse = await res.json()
      setFiles(data.files ?? [])
      setError(false)
    } catch {
      setError(true)
    } finally {
      setLoading(false)
    }
  }, [])

  // Initial fetch + interval
  useEffect(() => {
    fetchFiles()
    timerRef.current = setInterval(fetchFiles, intervalMs)
    return () => {
      if (timerRef.current) clearInterval(timerRef.current)
    }
  }, [fetchFiles, intervalMs])

  // Pause when tab hidden, resume + re-fetch on visible
  useEffect(() => {
    const handleVisibility = () => {
      if (document.hidden) {
        if (timerRef.current) {
          clearInterval(timerRef.current)
          timerRef.current = null
        }
      } else {
        fetchFiles()
        timerRef.current = setInterval(fetchFiles, intervalMs)
      }
    }
    document.addEventListener('visibilitychange', handleVisibility)
    return () => document.removeEventListener('visibilitychange', handleVisibility)
  }, [fetchFiles, intervalMs])

  // Re-fetch on window focus
  useEffect(() => {
    const handleFocus = () => fetchFiles()
    window.addEventListener('focus', handleFocus)
    return () => window.removeEventListener('focus', handleFocus)
  }, [fetchFiles])

  return { files, loading, error, refetch: fetchFiles }
}
