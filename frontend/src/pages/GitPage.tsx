import { useState } from 'react'
import { useParams, useNavigate } from 'react-router-dom'
import { WorkspaceShell } from '@/components/layout/WorkspaceShell'
import { useGitStatus } from '@/hooks/useGitStatus'
import { useGitFiles } from '@/hooks/useGitFiles'
import { GitHistoryTab } from '@/components/git/GitHistoryTab'
import { GitFileBrowser } from '@/components/git/GitFileBrowser'
import { cn } from '@/lib/utils'
import { ArrowLeft, Files, GitBranch } from 'lucide-react'
import type { GitFile } from '@/hooks/useGitFiles'

type GitTab = 'files' | 'history'

function severityDotClass(severity: 'green' | 'yellow' | 'red'): string {
  switch (severity) {
    case 'green': return 'bg-emerald-500 shadow-[0_0_4px_rgba(16,185,129,0.4)]'
    case 'yellow': return 'bg-amber-500 shadow-[0_0_4px_rgba(245,158,11,0.4)]'
    case 'red': return 'bg-red-500 shadow-[0_0_4px_rgba(239,68,68,0.4)]'
  }
}

function GitListPane({ onFileSelect, selectedPath }: { onFileSelect: (file: GitFile) => void; selectedPath: string | null }) {
  const { status, loading } = useGitStatus()
  const { files, loading: filesLoading, error: filesError, refetch: filesRefetch } = useGitFiles()
  const [activeTab, setActiveTab] = useState<GitTab>('history')

  const dotClass = status
    ? severityDotClass(status.severity)
    : 'bg-muted-foreground/30'

  const branchName = loading
    ? 'Loading...'
    : status?.branch ?? 'unknown'

  const summaryText = loading
    ? ''
    : status?.summary ?? ''

  const tabs: { key: GitTab; label: string }[] = [
    { key: 'history', label: 'History' },
    { key: 'files', label: 'Files' },
  ]

  return (
    <div className="flex flex-col h-full">
      {/* Branch header */}
      <div className="px-4 py-3 border-b border-border">
        <div className="flex items-center gap-2">
          <span className={cn('w-2 h-2 rounded-full shrink-0', dotClass)} />
          <span className="text-sm font-semibold truncate">{branchName}</span>
        </div>
        {summaryText && (
          <p className="text-xs text-muted-foreground mt-1">{summaryText}</p>
        )}
      </div>

      {/* Tab bar */}
      <div className="flex border-b border-border">
        {tabs.map((tab) => (
          <button
            key={tab.key}
            onClick={() => setActiveTab(tab.key)}
            className={cn(
              'px-4 py-2 text-xs font-medium border-b-2 transition-colors',
              activeTab === tab.key
                ? 'border-foreground text-foreground'
                : 'border-transparent text-muted-foreground hover:text-foreground',
            )}
          >
            {tab.label}
          </button>
        ))}
      </div>

      {/* Tab content */}
      <div className="flex-1 overflow-y-auto">
        {activeTab === 'history' && <GitHistoryTab />}
        {activeTab === 'files' && (
          <GitFileBrowser
            files={files}
            loading={filesLoading}
            error={filesError}
            onSelect={onFileSelect}
            selectedPath={selectedPath ?? undefined}
            onRetry={filesRefetch}
          />
        )}
      </div>
    </div>
  )
}

function GitDetailPane({ selectedPath, onBack }: { selectedPath: string | null; onBack: () => void }) {
  if (!selectedPath) {
    return (
      <div className="flex-1 flex flex-col items-center justify-center gap-2 text-muted-foreground/50">
        <Files className="w-8 h-8" />
        <p className="text-sm">Select a file to view details</p>
      </div>
    )
  }

  return (
    <div className="flex-1 flex flex-col overflow-hidden">
      <div className="px-4 py-3 border-b border-border flex items-center gap-2">
        <button
          onClick={onBack}
          className="lg:hidden p-1 rounded-md text-muted-foreground hover:text-foreground hover:bg-accent transition-colors"
          aria-label="Back to file list"
        >
          <ArrowLeft className="w-4 h-4" />
        </button>
        <GitBranch className="w-4 h-4 text-muted-foreground shrink-0" />
        <span className="text-sm font-mono truncate">{selectedPath}</span>
      </div>
      <div className="flex-1 flex items-center justify-center text-muted-foreground/50">
        <p className="text-sm">Diff viewer will appear here</p>
      </div>
    </div>
  )
}

export default function GitPage() {
  const params = useParams()
  const navigate = useNavigate()

  // The wildcard param captures everything after /git/
  const selectedPath = params['*'] || null

  const handleFileSelect = (file: GitFile) => {
    navigate(`/git/${file.path}`)
  }

  const handleBack = () => {
    navigate('/git')
  }

  return (
    <WorkspaceShell
      listPane={<GitListPane onFileSelect={handleFileSelect} selectedPath={selectedPath} />}
      detailPane={<GitDetailPane selectedPath={selectedPath} onBack={handleBack} />}
      showDetail={!!selectedPath}
    />
  )
}
