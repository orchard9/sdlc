import { cn } from '@/lib/utils'
import { MarkdownContent } from './MarkdownContent'

interface Props {
  filename: string
  content: string
  fullscreen?: boolean
}

export function ArtifactContent({ filename, content, fullscreen }: Props) {
  const ext = filename.split('.').pop()?.toLowerCase() ?? ''
  if (['md', 'markdown'].includes(ext)) {
    return <MarkdownContent content={content} />
  }
  if (['html', 'htm'].includes(ext)) {
    return (
      <iframe
        srcDoc={content}
        sandbox="allow-scripts"
        className={cn(
          'w-full border-0 rounded bg-white',
          fullscreen ? 'min-h-[60vh]' : 'min-h-64 max-h-80',
        )}
        title={filename}
      />
    )
  }
  const lang =
    ext === 'tsx' ? 'tsx'
    : ext === 'ts' ? 'typescript'
    : ext === 'jsx' ? 'jsx'
    : ext === 'js' ? 'javascript'
    : ext === 'json' ? 'json'
    : ext === 'rs' ? 'rust'
    : ext || 'text'
  return <MarkdownContent content={'```' + lang + '\n' + content + '\n```'} />
}
