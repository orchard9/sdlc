import { MarkdownContent } from '@/components/shared/MarkdownContent'

interface Props {
  name: string
  role: string
  content: string
  /** Highlight the owner's own messages differently */
  isOwner?: boolean
}

export function PartnerMessage({ name, role, content, isOwner }: Props) {
  return (
    <div className={isOwner ? 'my-3 border border-border/50 rounded-lg px-4 py-3 bg-muted/20' : 'my-3'}>
      <div className="flex items-baseline gap-2 mb-1.5">
        <span className={`text-sm font-bold tracking-wide ${isOwner ? 'text-primary' : 'text-foreground'}`}>
          {name}
        </span>
        <span className="text-xs text-muted-foreground/60">·</span>
        <span className={`text-xs font-medium ${isOwner ? 'text-primary/70' : 'text-primary/80'}`}>
          {role}
        </span>
      </div>
      <div className="text-sm text-foreground/80 leading-relaxed">
        <MarkdownContent content={content} />
      </div>
    </div>
  )
}
