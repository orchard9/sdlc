import ReactMarkdown from 'react-markdown'
import remarkGfm from 'remark-gfm'
import { cn } from '@/lib/utils'

interface CompactMarkdownProps {
  content: string
  className?: string
}

/**
 * Lightweight markdown renderer for activity feed cards and other tight spaces.
 *
 * Renders a minimal subset of markdown (bold, italic, inline code, links, lists,
 * code fences, blockquotes, strikethrough) without the TOC, raw toggle, mermaid,
 * or syntax highlighting of MarkdownContent.
 */
export function CompactMarkdown({ content, className }: CompactMarkdownProps) {
  if (!content || !content.trim()) return null

  return (
    <div className={cn('compact-md', className)}>
      <ReactMarkdown
        remarkPlugins={[remarkGfm]}
        components={{
          h1: ({ children }) => <p className="text-xs font-semibold text-foreground mb-1 last:mb-0">{children}</p>,
          h2: ({ children }) => <p className="text-xs font-semibold text-foreground mb-1 last:mb-0">{children}</p>,
          h3: ({ children }) => <p className="text-xs font-semibold text-foreground mb-1 last:mb-0">{children}</p>,
          p: ({ children }) => <p className="text-xs leading-relaxed mb-1 last:mb-0">{children}</p>,
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
          ul: ({ children }) => <ul className="list-disc pl-4 mb-1 space-y-0.5 text-xs">{children}</ul>,
          ol: ({ children }) => <ol className="list-decimal pl-4 mb-1 space-y-0.5 text-xs">{children}</ol>,
          li: ({ children }) => <li className="text-xs leading-relaxed">{children}</li>,
          blockquote: ({ children }) => (
            <blockquote className="border-l-2 border-border pl-2 my-1 text-muted-foreground italic text-xs">
              {children}
            </blockquote>
          ),
          code: ({ children, className: codeClass, node }) => {
            const isBlock = node?.position?.start.line !== node?.position?.end.line
            if (codeClass || isBlock) {
              return (
                <pre className="text-[10px] font-mono bg-muted/40 border border-border/40 rounded p-2 whitespace-pre-wrap overflow-x-auto mb-1 last:mb-0">
                  <code>{children}</code>
                </pre>
              )
            }
            return (
              <code className="text-[10px] font-mono bg-muted/60 border border-border/50 px-1 py-0.5 rounded text-muted-foreground">
                {children}
              </code>
            )
          },
          pre: ({ children }) => <div className="mb-1 last:mb-0">{children}</div>,
          hr: () => <hr className="border-border my-2" />,
          table: ({ children }) => (
            <div className="overflow-x-auto mb-1">
              <table className="w-full text-xs border-collapse">{children}</table>
            </div>
          ),
          thead: ({ children }) => <thead className="border-b border-border">{children}</thead>,
          tbody: ({ children }) => <tbody>{children}</tbody>,
          tr: ({ children }) => <tr className="border-b border-border/50 last:border-0">{children}</tr>,
          th: ({ children }) => <th className="text-left px-2 py-1 text-[10px] font-semibold text-muted-foreground">{children}</th>,
          td: ({ children }) => <td className="px-2 py-1 text-xs text-foreground">{children}</td>,
        }}
      >
        {content}
      </ReactMarkdown>
    </div>
  )
}
