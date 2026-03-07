import { useEffect, useRef } from 'react'
import type {
  HubProjectEntry,
  HubSseEvent,
  FleetInstance,
  FleetAgentSummary,
  ProvisionEntry,
  HubActivityEntry,
} from '@/lib/types'

/** Compute hub project status based on last_seen age (mirrors server-side logic). */
function statusForAge(lastSeen: string): 'online' | 'stale' | 'offline' {
  const ageMs = Date.now() - new Date(lastSeen).getTime()
  const ageSecs = ageMs / 1000
  if (ageSecs < 30) return 'online'
  if (ageSecs < 90) return 'stale'
  return 'offline'
}

export interface HubSseCallbacks {
  onProjectUpdated: (project: HubProjectEntry) => void
  onProjectRemoved: (url: string) => void
  onFleetUpdated?: (instance: FleetInstance) => void
  onFleetProvisioned?: (instance: FleetInstance) => void
  onFleetAgentStatus?: (summary: FleetAgentSummary) => void
  onProvisionUpdated?: (provision: ProvisionEntry) => void
  onActivityAppended?: (activity: HubActivityEntry) => void
}

/**
 * Subscribe to /api/hub/events SSE stream.
 *
 * Separate from the main SseContext since:
 * - Different endpoint (/api/hub/events vs /api/events)
 * - Different event schema (hub-specific payloads)
 * - Only active in hub mode
 *
 * Also runs a 15-second interval to recompute project statuses
 * client-side from last_seen, so dots update smoothly between server events.
 */
export function useHubSSE(
  callbacks: HubSseCallbacks,
  onRecompute: (updater: (projects: HubProjectEntry[]) => HubProjectEntry[]) => void,
) {
  const callbacksRef = useRef(callbacks)
  const onRecomputeRef = useRef(onRecompute)

  useEffect(() => {
    callbacksRef.current = callbacks
    onRecomputeRef.current = onRecompute
  })

  // SSE connection
  useEffect(() => {
    let active = true

    async function connect() {
      while (active) {
        try {
          const response = await fetch('/api/hub/events', {
            headers: { Accept: 'text/event-stream' },
          })

          if (!response.ok || !response.body) {
            await new Promise(r => setTimeout(r, 3000))
            continue
          }

          const reader = response.body.getReader()
          const decoder = new TextDecoder()
          let buffer = ''
          let currentType = 'message'
          let currentData = ''

          while (active) {
            const { done, value } = await reader.read()
            if (done) {
              await new Promise(r => setTimeout(r, 2000))
              break
            }

            buffer += decoder.decode(value, { stream: true })
            const lines = buffer.split('\n')
            buffer = lines.pop()!

            for (const line of lines) {
              if (line.startsWith('event:')) {
                currentType = line.slice(6).trim()
              } else if (line.startsWith('data:')) {
                currentData = line.slice(5).trim()
              } else if (line === '' || line === '\r') {
                if (currentData && currentType === 'hub') {
                  try {
                    const event = JSON.parse(currentData) as HubSseEvent
                    if (event.type === 'project_updated' && event.project) {
                      callbacksRef.current.onProjectUpdated(event.project)
                    } else if (event.type === 'project_removed' && event.url) {
                      callbacksRef.current.onProjectRemoved(event.url)
                    } else if (event.type === 'fleet_updated' && event.instance) {
                      callbacksRef.current.onFleetUpdated?.(event.instance)
                    } else if (event.type === 'fleet_provisioned' && event.instance) {
                      callbacksRef.current.onFleetProvisioned?.(event.instance)
                    } else if (event.type === 'fleet_agent_status' && event.agent_summary) {
                      callbacksRef.current.onFleetAgentStatus?.(event.agent_summary)
                    } else if (event.type === 'provision_updated' && event.provision) {
                      callbacksRef.current.onProvisionUpdated?.(event.provision)
                    } else if (event.type === 'activity_appended' && event.activity) {
                      callbacksRef.current.onActivityAppended?.(event.activity)
                    }
                  } catch { /* malformed */ }
                }
                currentType = 'message'
                currentData = ''
              }
            }
          }
        } catch {
          if (!active) break
          await new Promise(r => setTimeout(r, 3000))
        }
      }
    }

    connect()
    return () => { active = false }
  }, [])

  // 15-second recompute interval for client-side status updates
  useEffect(() => {
    const id = setInterval(() => {
      onRecomputeRef.current(projects =>
        projects.map(p => ({ ...p, status: statusForAge(p.last_seen) }))
      )
    }, 15_000)
    return () => clearInterval(id)
  }, [])
}
