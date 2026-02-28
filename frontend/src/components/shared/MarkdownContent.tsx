import { useEffect, useMemo, useRef, useState } from 'react'
import ReactMarkdown from 'react-markdown'
import remarkGfm from 'remark-gfm'
import SyntaxHighlighter from 'react-syntax-highlighter'
import { atomOneDark } from 'react-syntax-highlighter/dist/esm/styles/hljs'
import mermaid from 'mermaid'
import { cn } from '@/lib/utils'

interface MarkdownContentProps {
  content: string
  className?: string
}

function MermaidBlock({ chart }: { chart: string }) {
  const id = useMemo(() => `mmd-${Math.random().toString(36).slice(2)}`, [])
  const ref = useRef<HTMLDivElement>(null)
  useEffect(() => {
    mermaid.initialize({ startOnLoad: false, theme: 'dark' })
    mermaid.render(id, chart)
      .then(({ svg }) => { if (ref.current) ref.current.innerHTML = svg })
      .catch(() => { if (ref.current) ref.current.textContent = chart })
  }, [chart, id])
  return <div ref={ref} className="my-3 flex justify-start" />
}

export function MarkdownContent({ content, className }: MarkdownContentProps) {
  const [raw, setRaw] = useState(false)

  return (
    <div className={cn('relative', className)}>
      <button
        onClick={() => setRaw(prev => !prev)}
        className="absolute top-0 right-0 px-2 py-0.5 text-xs rounded-md bg-muted text-muted-foreground hover:text-foreground transition-colors z-10"
      >
        {raw ? 'Rendered' : 'Raw'}
      </button>

      {raw ? (
        <pre className="text-xs whitespace-pre-wrap text-muted-foreground font-mono pt-6">
          {content}
        </pre>
      ) : (
        <div className="pt-6 text-sm text-foreground">
          <ReactMarkdown
            remarkPlugins={[remarkGfm]}
            components={{
              h1: ({ children }) => <h1 className="text-xl font-bold text-foreground mt-4 mb-2 first:mt-0">{children}</h1>,
              h2: ({ children }) => <h2 className="text-lg font-semibold text-foreground mt-4 mb-2 first:mt-0">{children}</h2>,
              h3: ({ children }) => <h3 className="text-base font-semibold text-foreground mt-3 mb-1.5 first:mt-0">{children}</h3>,
              p:  ({ children }) => <p className="text-sm text-foreground leading-relaxed mb-3 last:mb-0">{children}</p>,
              ul: ({ children }) => <ul className="list-disc pl-5 mb-3 space-y-1 text-sm text-foreground">{children}</ul>,
              ol: ({ children }) => <ol className="list-decimal pl-5 mb-3 space-y-1 text-sm text-foreground">{children}</ol>,
              li: ({ children }) => <li className="text-sm text-foreground leading-relaxed">{children}</li>,
              code: ({ children, className: codeClass, node }) => {
                const lang = codeClass?.replace('language-', '') ?? ''
                const isBlock = node?.position?.start.line !== node?.position?.end.line
                if (lang === 'mermaid') return <MermaidBlock chart={String(children).trim()} />
                if (lang) return (
                  <SyntaxHighlighter
                    language={lang}
                    style={atomOneDark}
                    customStyle={{ margin: 0, borderRadius: '0.5rem', fontSize: '0.75rem', whiteSpace: 'pre-wrap' }}
                    wrapLongLines={false}
                  >
                    {String(children).trim()}
                  </SyntaxHighlighter>
                )
                if (isBlock) return (
                  <pre className="text-xs font-mono bg-muted/60 border border-border/50 rounded p-3 whitespace-pre-wrap overflow-x-auto text-muted-foreground">
                    <code>{children}</code>
                  </pre>
                )
                return <code className="text-xs font-mono bg-muted/60 border border-border/50 px-1 py-0.5 rounded text-muted-foreground">{children}</code>
              },
              pre: ({ children }) => <div className="mb-3">{children}</div>,
              blockquote: ({ children }) => <blockquote className="border-l-2 border-border pl-3 my-3 text-muted-foreground italic">{children}</blockquote>,
              hr: () => <hr className="border-border my-4" />,
              strong: ({ children }) => <strong className="font-semibold text-foreground">{children}</strong>,
              em: ({ children }) => <em className="italic text-muted-foreground">{children}</em>,
              del: ({ children }) => <del className="line-through text-muted-foreground">{children}</del>,
              a: ({ href, children }) => (
                <a
                  href={href}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="text-primary underline underline-offset-2 hover:opacity-80 transition-opacity"
                >
                  {children}
                </a>
              ),
              table: ({ children }) => (
                <div className="overflow-x-auto mb-3">
                  <table className="w-full text-sm border-collapse">{children}</table>
                </div>
              ),
              thead: ({ children }) => <thead className="border-b border-border">{children}</thead>,
              tbody: ({ children }) => <tbody>{children}</tbody>,
              tr: ({ children }) => <tr className="border-b border-border/50 last:border-0">{children}</tr>,
              th: ({ children }) => <th className="text-left px-3 py-2 text-xs font-semibold text-muted-foreground uppercase tracking-wider">{children}</th>,
              td: ({ children }) => <td className="px-3 py-2 text-sm text-foreground">{children}</td>,
            }}
          >
            {content}
          </ReactMarkdown>
        </div>
      )}
    </div>
  )
}
