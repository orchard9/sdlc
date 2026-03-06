import type { PairedAssistantText } from '@/lib/types'
import { CompactMarkdown } from '@/components/shared/CompactMarkdown'

interface AssistantTextBlockProps {
  event: PairedAssistantText
}

export function AssistantTextBlock({ event }: AssistantTextBlockProps) {
  if (!event.text.trim()) return null

  return (
    <div className="py-1">
      <CompactMarkdown content={event.text} className="text-foreground/90" />
    </div>
  )
}
