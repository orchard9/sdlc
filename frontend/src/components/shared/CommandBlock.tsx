import { CopyButton } from './CopyButton'

interface CommandBlockProps {
  cmd: string
}

export function CommandBlock({ cmd }: CommandBlockProps) {
  return (
    <div className="flex items-center gap-2">
      <code className="flex-1 text-xs font-mono bg-muted/60 border border-border/50 px-3 py-1.5 rounded-lg text-muted-foreground select-all">
        {cmd}
      </code>
      <CopyButton text={cmd} />
    </div>
  )
}
