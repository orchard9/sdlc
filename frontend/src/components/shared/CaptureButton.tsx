import { useRef, useState } from 'react'
import { Image, Check, Download, Loader2 } from 'lucide-react'
import { cn } from '@/lib/utils'

type CaptureState = 'idle' | 'capturing' | 'done'

interface CaptureButtonProps {
  targetRef: React.RefObject<HTMLElement | null>
  mode: 'copy' | 'download'
  label?: string
  filename?: string
  title?: string
  className?: string
}

async function captureCanvas(el: HTMLElement): Promise<HTMLCanvasElement> {
  const { default: html2canvas } = await import('html2canvas')
  return html2canvas(el, { useCORS: true, backgroundColor: '#0e0f14' })
}

function downloadCanvas(canvas: HTMLCanvasElement, name: string) {
  const a = document.createElement('a')
  a.href = canvas.toDataURL('image/png')
  a.download = name
  a.click()
}

async function copyCanvas(canvas: HTMLCanvasElement): Promise<void> {
  const blob = await new Promise<Blob>((resolve, reject) =>
    canvas.toBlob(b => (b ? resolve(b) : reject(new Error('toBlob failed'))), 'image/png')
  )
  await navigator.clipboard.write([new ClipboardItem({ 'image/png': blob })])
}

export function CaptureButton({
  targetRef,
  mode,
  label,
  filename = 'artifact.png',
  title,
  className,
}: CaptureButtonProps) {
  const [state, setState] = useState<CaptureState>('idle')
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null)

  async function handleClick(e: React.MouseEvent) {
    e.stopPropagation()
    if (state !== 'idle' || !targetRef.current) return

    setState('capturing')
    try {
      const canvas = await captureCanvas(targetRef.current)
      if (mode === 'copy') {
        try {
          await copyCanvas(canvas)
        } catch {
          // clipboard write denied — fall back to download silently
          downloadCanvas(canvas, filename)
        }
      } else {
        downloadCanvas(canvas, filename)
      }
    } catch {
      // capture itself failed — no-op, revert
    }
    setState('done')
    if (timerRef.current) clearTimeout(timerRef.current)
    timerRef.current = setTimeout(() => setState('idle'), 2000)
  }

  const defaultClass = cn(
    'shrink-0 flex items-center gap-1 p-1.5 rounded-lg border transition-colors',
    state === 'done'
      ? 'border-green-800/50 bg-green-950/60 text-green-400 cursor-default'
      : state === 'capturing'
        ? 'border-border/50 bg-muted/60 text-muted-foreground/60 cursor-wait'
        : 'border-border/50 bg-muted/60 hover:bg-muted text-muted-foreground hover:text-foreground cursor-pointer',
  )

  const DefaultIcon = mode === 'copy' ? Image : Download

  return (
    <button
      onClick={handleClick}
      disabled={state !== 'idle'}
      className={className ?? defaultClass}
      title={title ?? (mode === 'copy' ? 'Copy as image' : 'Download PNG')}
    >
      {state === 'capturing' && <Loader2 className="w-3.5 h-3.5 animate-spin" />}
      {state === 'done' && <Check className="w-3.5 h-3.5" />}
      {state === 'idle' && <DefaultIcon className="w-3.5 h-3.5" />}
      {label && (
        <span className="text-[10px] font-medium">
          {state === 'done' ? (mode === 'copy' ? 'Copied' : 'Saved') : state === 'capturing' ? '…' : label}
        </span>
      )}
    </button>
  )
}
