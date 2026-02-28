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

  diagnose: (description: string) =>
    request<import('@/lib/types').DiagnoseResult>('/api/diagnose', {
      method: 'POST',
      body: JSON.stringify({ description }),
    }),

  startRun: (slug: string, context?: string) =>
    request<{ status: string; message: string }>(`/api/run/${slug}`, {
      method: 'POST',
      body: context ? JSON.stringify({ context }) : undefined,
    }),
  stopRun: (slug: string) =>
    request<{ status: string; message: string }>(`/api/run/${slug}/stop`, { method: 'POST' }),

  startMilestoneUat: (slug: string) =>
    request<{ status: string; message: string }>(`/api/milestone/${encodeURIComponent(slug)}/uat`, { method: 'POST' }),
  stopMilestoneUat: (slug: string) =>
    request<{ status: string; message: string }>(`/api/milestone/${encodeURIComponent(slug)}/uat/stop`, { method: 'POST' }),

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

  // Project prepare
  getProjectPhase: () => request<import('@/lib/types').ProjectPhase>('/api/project/phase'),
  getProjectPrepare: (milestone?: string) =>
    request<import('@/lib/types').PrepareResult>(`/api/project/prepare${milestone ? `?milestone=${encodeURIComponent(milestone)}` : ''}`),

  getVision: () => request<{ content: string; exists: boolean }>('/api/vision'),
  putVision: (content: string) =>
    request('/api/vision', { method: 'PUT', body: JSON.stringify({ content }) }),

  getArchitecture: () => request<{ content: string; exists: boolean }>('/api/architecture'),
  putArchitecture: (content: string) =>
    request('/api/architecture', { method: 'PUT', body: JSON.stringify({ content }) }),
  runVisionAlign: () => request<{ status: string; run_id: string }>('/api/vision/run', { method: 'POST' }),
  runArchitectureAlign: () => request<{ status: string; run_id: string }>('/api/architecture/run', { method: 'POST' }),

  // Roadmap / Ponder
  getRoadmap: () => request<import('@/lib/types').PonderSummary[]>('/api/roadmap'),
  getPonderEntry: (slug: string) => request<import('@/lib/types').PonderDetail>(`/api/roadmap/${slug}`),
  createPonderEntry: (data: { slug: string; title: string; brief?: string }) =>
    request<{ slug: string; title: string; status: string }>('/api/roadmap', { method: 'POST', body: JSON.stringify(data) }),
  updatePonderEntry: (slug: string, data: Partial<{ title: string; status: import('@/lib/types').PonderStatus; tags: string[]; committed_to: string[] }>) =>
    request<{ slug: string; title: string; status: string; tags: string[]; committed_to: string[] }>(`/api/roadmap/${slug}`, { method: 'PUT', body: JSON.stringify(data) }),
  capturePonderArtifact: (slug: string, data: { filename: string; content: string }) =>
    request<void>(`/api/roadmap/${slug}/capture`, { method: 'POST', body: JSON.stringify(data) }),
  getPonderSessions: (slug: string) =>
    request<import('@/lib/types').SessionMeta[]>(`/api/roadmap/${slug}/sessions`),
  getPonderSession: (slug: string, n: number) =>
    request<import('@/lib/types').SessionContent>(`/api/roadmap/${slug}/sessions/${n}`),

  // Ponder chat — start / stop agent sessions
  startPonderChat: (slug: string, message?: string) =>
    request<import('@/lib/types').PonderChatResponse>(`/api/ponder/${slug}/chat`, {
      method: 'POST',
      body: JSON.stringify({ message: message ?? null }),
    }),
  stopPonderChat: (slug: string) =>
    request<void>(`/api/ponder/${slug}/chat/current`, { method: 'DELETE' }),
  commitPonder: (slug: string) =>
    request<{ status: string; run_id: string }>(`/api/ponder/${slug}/commit`, { method: 'POST' }),

  // Investigations
  getInvestigations: (kind?: import('@/lib/types').InvestigationKind) =>
    request<import('@/lib/types').InvestigationSummary[]>(
      `/api/investigations${kind ? `?kind=${encodeURIComponent(kind)}` : ''}`
    ),
  getInvestigation: (slug: string) =>
    request<import('@/lib/types').InvestigationDetail>(`/api/investigations/${slug}`),
  createInvestigation: (data: { slug: string; title: string; kind: import('@/lib/types').InvestigationKind; context?: string }) =>
    request<{ slug: string; title: string; kind: string; phase: string; status: string }>(
      '/api/investigations', { method: 'POST', body: JSON.stringify(data) }
    ),
  updateInvestigation: (slug: string, data: Partial<{ phase: string; status: string; title: string; scope: string; confidence: number; output_type: string; output_ref: string }>) =>
    request<{ slug: string; phase: string; status: string }>(
      `/api/investigations/${slug}`, { method: 'PUT', body: JSON.stringify(data) }
    ),
  getInvestigationSessions: (slug: string) =>
    request<import('@/lib/types').SessionMeta[]>(`/api/investigations/${slug}/sessions`),
  getInvestigationSession: (slug: string, n: number) =>
    request<import('@/lib/types').SessionContent>(`/api/investigations/${slug}/sessions/${n}`),
  startInvestigationChat: (slug: string, message?: string) =>
    request<import('@/lib/types').InvestigationChatResponse>(`/api/investigation/${slug}/chat`, {
      method: 'POST',
      body: JSON.stringify({ message: message ?? null }),
    }),
  stopInvestigationChat: (slug: string) =>
    request<void>(`/api/investigation/${slug}/chat/current`, { method: 'DELETE' }),

  // Run history
  getRuns: () => request<import('@/lib/types').RunRecord[]>('/api/runs'),
  getRun: (id: string) => request<import('@/lib/types').RunRecord & { events: import('@/lib/types').AgentEvent[] }>(`/api/runs/${id}`),

  // Escalations
  getEscalations: (status?: string) =>
    request<import('@/lib/types').EscalationDetail[]>(
      `/api/escalations${status ? `?status=${encodeURIComponent(status)}` : ''}`
    ),
  getEscalation: (id: string) =>
    request<import('@/lib/types').EscalationDetail>(`/api/escalations/${encodeURIComponent(id)}`),
  createEscalation: (data: { kind: string; title: string; context: string; source_feature?: string }) =>
    request<import('@/lib/types').EscalationDetail>('/api/escalations', {
      method: 'POST',
      body: JSON.stringify(data),
    }),
  resolveEscalation: (id: string, resolution: string) =>
    request<import('@/lib/types').EscalationDetail>(`/api/escalations/${encodeURIComponent(id)}/resolve`, {
      method: 'POST',
      body: JSON.stringify({ resolution }),
    }),

  // Tools
  listTools: () => request<import('@/lib/types').ToolMeta[]>('/api/tools'),
  getTool: (name: string) => request<import('@/lib/types').ToolMeta>(`/api/tools/${encodeURIComponent(name)}`),
  runTool: (name: string, input: unknown) =>
    request<import('@/lib/types').ToolResult>(`/api/tools/${encodeURIComponent(name)}/run`, {
      method: 'POST',
      body: JSON.stringify(input),
    }),
  setupTool: (name: string) =>
    request<import('@/lib/types').ToolResult>(`/api/tools/${encodeURIComponent(name)}/setup`, {
      method: 'POST',
      body: '{}',
    }),
  answerAma: (
    question: string,
    sources: import('@/lib/types').AmaSource[],
    opts?: { turnIndex?: number; threadContext?: string }
  ) =>
    request<{ status: string; run_id: string; run_key: string }>('/api/tools/ama/answer', {
      method: 'POST',
      body: JSON.stringify({
        question,
        sources,
        turn_index: opts?.turnIndex ?? 0,
        thread_context: opts?.threadContext ?? null,
      }),
    }),
  reconfigureQualityGates: () =>
    request<{ status: string; run_id: string; run_key: string }>('/api/tools/quality-check/reconfigure', {
      method: 'POST',
    }),
  fixQualityIssues: (failedChecks: import('@/lib/types').CheckResult[]) =>
    request<{ status: string; run_id: string; run_key: string }>('/api/tools/quality-check/fix', {
      method: 'POST',
      body: JSON.stringify({ failed_checks: failedChecks }),
    }),

  // Feedback
  getFeedback: () => request<import('@/lib/types').FeedbackNote[]>('/api/feedback'),
  addFeedbackNote: (content: string) =>
    request<import('@/lib/types').FeedbackNote>('/api/feedback', { method: 'POST', body: JSON.stringify({ content }) }),
  deleteFeedbackNote: (id: string) =>
    request<{ deleted: boolean }>(`/api/feedback/${encodeURIComponent(id)}`, { method: 'DELETE' }),
  submitFeedbackToPonder: () =>
    request<{ slug: string; note_count: number }>('/api/feedback/to-ponder', { method: 'POST' }),

  // SDLC tunnel (exposes this UI)
  getTunnel: () => request<import('@/lib/types').TunnelStatus>('/api/tunnel'),
  startTunnel: () => request<import('@/lib/types').TunnelStatus>('/api/tunnel', { method: 'POST' }),
  stopTunnel: () => request<import('@/lib/types').TunnelStatus>('/api/tunnel', { method: 'DELETE' }),

  // App tunnel (exposes user's project dev server on a configurable port)
  getAppTunnel: () => request<import('@/lib/types').AppTunnelStatus>('/api/app-tunnel'),
  startAppTunnel: (port: number) =>
    request<import('@/lib/types').AppTunnelStatus>('/api/app-tunnel', {
      method: 'POST',
      body: JSON.stringify({ port }),
    }),
  stopAppTunnel: () => request<import('@/lib/types').AppTunnelStatus>('/api/app-tunnel', { method: 'DELETE' }),
  setAppPort: (port: number) =>
    request<import('@/lib/types').AppTunnelStatus>('/api/app-tunnel/port', {
      method: 'PUT',
      body: JSON.stringify({ port }),
    }),

  // Agents (Claude agent definitions from ~/.claude/agents/)
  getAgents: () => request<import('@/lib/types').AgentDefinition[]>('/api/agents'),
  getAgent: (name: string) => request<import('@/lib/types').AgentDefinition>(`/api/agents/${encodeURIComponent(name)}`),

  // Secrets (metadata only — decryption is CLI-only)
  getSecretsStatus: () => request<{ key_count: number; env_count: number }>('/api/secrets/status'),
  getSecretsKeys: () => request<import('@/lib/types').SecretsKey[]>('/api/secrets/keys'),
  addSecretsKey: (body: { name: string; public_key: string }) =>
    request('/api/secrets/keys', { method: 'POST', body: JSON.stringify(body) }),
  removeSecretsKey: (name: string) =>
    request(`/api/secrets/keys/${encodeURIComponent(name)}`, { method: 'DELETE' }),
  getSecretsEnvs: () => request<import('@/lib/types').SecretsEnvMeta[]>('/api/secrets/envs'),
  deleteSecretsEnv: (name: string) =>
    request(`/api/secrets/envs/${encodeURIComponent(name)}`, { method: 'DELETE' }),
}
