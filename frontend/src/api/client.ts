const BASE = ''

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const res = await fetch(`${BASE}${path}`, {
    headers: { 'Content-Type': 'application/json', ...init?.headers },
    ...init,
  })
  if (!res.ok) {
    const body = await res.json().catch(() => ({ error: res.statusText }))
    throw new Error(body.error || res.statusText)
  }
  return res.json()
}

export const api = {
  getState: () => request<import('@/lib/types').ProjectState>('/api/state'),
  getFeatures: () => request<import('@/lib/types').FeatureSummary[]>('/api/features'),
  getFeature: (slug: string) => request<import('@/lib/types').FeatureDetail>(`/api/features/${slug}`),
  getFeatureNext: (slug: string) => request<import('@/lib/types').Classification>(`/api/features/${slug}/next`),
  createFeature: (body: { slug: string; title: string; description?: string }) =>
    request('/api/features', { method: 'POST', body: JSON.stringify(body) }),
  transitionFeature: (slug: string, phase: string) =>
    request(`/api/features/${slug}/transition`, { method: 'POST', body: JSON.stringify({ phase }) }),

  getMilestones: () => request<import('@/lib/types').MilestoneSummary[]>('/api/milestones'),
  getMilestone: (slug: string) => request<import('@/lib/types').MilestoneDetail>(`/api/milestones/${slug}`),
  reviewMilestone: (slug: string) => request<import('@/lib/types').MilestoneReview>(`/api/milestones/${slug}/review`),
  createMilestone: (body: { slug: string; title: string }) =>
    request('/api/milestones', { method: 'POST', body: JSON.stringify(body) }),
  addFeatureToMilestone: (milestoneSlug: string, featureSlug: string) =>
    request(`/api/milestones/${milestoneSlug}/features`, {
      method: 'POST',
      body: JSON.stringify({ feature_slug: featureSlug }),
    }),
  reorderMilestoneFeatures: (milestoneSlug: string, features: string[]) =>
    request<import('@/lib/types').MilestoneDetail>(`/api/milestones/${milestoneSlug}/features/order`, {
      method: 'PUT', body: JSON.stringify({ features })
    }),

  getArtifact: (slug: string, type_: string) =>
    request<import('@/lib/types').Artifact>(`/api/artifacts/${slug}/${type_}`),
  approveArtifact: (slug: string, type_: string, by?: string) =>
    request(`/api/artifacts/${slug}/${type_}/approve`, { method: 'POST', body: JSON.stringify({ by }) }),
  rejectArtifact: (slug: string, type_: string, reason?: string) =>
    request(`/api/artifacts/${slug}/${type_}/reject`, { method: 'POST', body: JSON.stringify({ reason }) }),
  waiveArtifact: (slug: string, type_: string, reason?: string) =>
    request(`/api/artifacts/${slug}/${type_}/waive`, { method: 'POST', body: JSON.stringify({ reason }) }),

  addTask: (slug: string, title: string) =>
    request(`/api/features/${slug}/tasks`, { method: 'POST', body: JSON.stringify({ title }) }),
  startTask: (slug: string, taskId: string) =>
    request(`/api/features/${slug}/tasks/${taskId}/start`, { method: 'POST' }),
  completeTask: (slug: string, taskId: string) =>
    request(`/api/features/${slug}/tasks/${taskId}/complete`, { method: 'POST' }),

  addComment: (slug: string, body: string, flag?: string, by?: string) =>
    request(`/api/features/${slug}/comments`, { method: 'POST', body: JSON.stringify({ body, flag, by }) }),

  getConfig: () => request<import('@/lib/types').ProjectConfig>('/api/config'),

  querySearch: (q: string, limit = 10) =>
    request<import('@/lib/types').QuerySearchResponse>(`/api/query/search?q=${encodeURIComponent(q)}&limit=${limit}`),
  querySearchTasks: (q: string, limit = 10) =>
    request<import('@/lib/types').QueryTaskSearchResponse>(`/api/query/search-tasks?q=${encodeURIComponent(q)}&limit=${limit}`),
  queryBlocked: () =>
    request<import('@/lib/types').QueryBlockedItem[]>('/api/query/blocked'),
  queryReady: (phase?: string) =>
    request<import('@/lib/types').QueryReadyItem[]>(`/api/query/ready${phase ? `?phase=${encodeURIComponent(phase)}` : ''}`),
  queryNeedsApproval: () =>
    request<import('@/lib/types').QueryNeedsApprovalItem[]>('/api/query/needs-approval'),

  getVision: () => request<{ content: string; exists: boolean }>('/api/vision'),
  putVision: (content: string) =>
    request('/api/vision', { method: 'PUT', body: JSON.stringify({ content }) }),

}
