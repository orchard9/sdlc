import type { PairedAssistantText } from '@/lib/types'

interface AssistantTextBlockProps {
  event: PairedAssistantText
}

export function AssistantTextBlock({ event }: AssistantTextBlockProps) {
  if (!event.text.trim()) return null

  return (
    <div className="py-1">
      <p className="text-xs text-foreground/90 whitespace-pre-wrap leading-relaxed">
        {event.text}
      </p>
    </div>
  )
}
