import { useMemo, useState } from 'react'
import { FileText, ChevronDown, ChevronUp } from 'lucide-react'
import { parseSession } from '@/lib/parseSession'
import { ToolCallBlock } from './ToolCallBlock'
import { PartnerMessage } from './PartnerMessage'
import { MarkdownContent } from '@/components/shared/MarkdownContent'
import { ArtifactContent } from '@/components/shared/ArtifactContent'
import { cn } from '@/lib/utils'
import type { SessionContent, PonderArtifact } from '@/lib/types'

interface Props {
  session: SessionContent
  ownerName?: string | null
  artifacts?: PonderArtifact[]
}

function relativeDate(iso: string | null): string {
  if (!iso) return ''
  const d = new Date(iso)
  if (isNaN(d.getTime())) return ''
  const diff = Date.now() - d.getTime()
  const secs = Math.floor(diff / 1000)
  if (secs < 60) return 'just now'
  const mins = Math.floor(secs / 60)
  if (mins < 60) return `${mins}m ago`
  const hours = Math.floor(mins / 60)
  if (hours < 24) return `${hours}h ago`
  const days = Math.floor(hours / 24)
  if (days === 1) return 'yesterday'
  if (days < 7) return `${days} days ago`
  return d.toLocaleDateString()
}

export function SessionBlock({ session, ownerName, artifacts = [] }: Props) {
  const events = useMemo(() => parseSession(session.content), [session.content])
  const dateLabel = relativeDate(session.timestamp)
  const [expandedFilename, setExpandedFilename] = useState<string | null>(null)

  return (
    <div className="mb-8">
      {/* Session header divider */}
      <div className="flex items-center gap-3 mb-5">
        <div className="flex-1 h-px bg-border/40" />
        <span className="text-xs text-muted-foreground/50 font-medium whitespace-nowrap">
          Session {session.session}{dateLabel ? `  ·  ${dateLabel}` : ''}
        </span>
        <div className="flex-1 h-px bg-border/40" />
      </div>

      {/* Events */}
      <div className="space-y-0.5">
        {events.map((event, idx) => {
          switch (event.kind) {
            case 'tool':
              return <ToolCallBlock key={idx} tool={event.tool} summary={event.summary} />

            case 'artifact': {
              const workspaceArtifact = artifacts.find(a => a.filename === event.filename)
              const hasContent = workspaceArtifact?.content != null
              const isExpanded = expandedFilename === event.filename
              return (
                <div
                  key={idx}
                  className="my-2 border border-border/50 rounded-lg overflow-hidden text-xs"
                >
                  <button
                    onClick={() => hasContent && setExpandedFilename(isExpanded ? null : event.filename)}
                    className={cn(
                      'w-full flex items-center gap-2 px-3 py-2 bg-muted/30 text-left',
                      hasContent && 'hover:bg-muted/50 transition-colors',
                    )}
                  >
                    <FileText className="w-3.5 h-3.5 text-muted-foreground/60 shrink-0" />
                    <span className="font-mono text-foreground/80 flex-1 truncate">
                      {event.filename}
                    </span>
                    <span className="text-muted-foreground/40">written</span>
                    {hasContent && (
                      isExpanded
                        ? <ChevronUp className="w-3 h-3 text-muted-foreground/40 shrink-0" />
                        : <ChevronDown className="w-3 h-3 text-muted-foreground/40 shrink-0" />
                    )}
                  </button>
                  {event.summary && !isExpanded && (
                    <div className="px-3 py-1.5 text-muted-foreground/60 font-mono truncate border-t border-border/30">
                      {event.summary.split('\n')[0]}
                    </div>
                  )}
                  {isExpanded && workspaceArtifact?.content && (
                    <div className="border-t border-border/30 overflow-auto max-h-72 px-3 py-2">
                      <ArtifactContent filename={event.filename} content={workspaceArtifact.content} />
                    </div>
                  )}
                </div>
              )
            }

            case 'partner': {
              const isOwner = ownerName
                ? event.name.toLowerCase() === ownerName.split(' ')[0].toLowerCase() &&
                  event.role.toLowerCase().includes('owner')
                : false
              return (
                <PartnerMessage
                  key={idx}
                  name={event.name}
                  role={event.role}
                  content={event.content}
                  isOwner={isOwner}
                />
              )
            }

            case 'recruited':
              return (
                <div key={idx} className="flex items-center gap-2 my-2 text-xs text-muted-foreground/60">
                  <span className="text-primary/60">→</span>
                  <span>
                    Recruited:{' '}
                    <span className="font-semibold text-foreground/70">{event.name}</span>
                    <span className="text-muted-foreground/50"> · {event.role}</span>
                  </span>
                </div>
              )

            case 'decision':
              return (
                <div key={idx} className="flex items-start gap-2 my-1.5 text-sm">
                  <span className="text-amber-400/80 font-bold shrink-0 mt-0.5">⚑</span>
                  <span className="text-foreground/80">{event.content}</span>
                </div>
              )

            case 'question':
              return (
                <div key={idx} className="flex items-start gap-2 my-1.5 text-sm">
                  <span className="text-blue-400/80 font-bold shrink-0 mt-0.5">?</span>
                  <span className="text-foreground/70 italic">{event.content}</span>
                </div>
              )

            case 'sketch':
              return (
                <pre
                  key={idx}
                  className="my-3 p-3 bg-muted/40 border border-border/40 rounded-lg text-xs font-mono text-foreground/70 overflow-x-auto leading-relaxed"
                >
                  {event.content}
                </pre>
              )

            case 'narrative':
              return (
                <div key={idx} className="my-2 text-sm text-foreground/75 leading-relaxed">
                  <MarkdownContent content={event.content} />
                </div>
              )

            default:
              return null
          }
        })}
      </div>
    </div>
  )
}
