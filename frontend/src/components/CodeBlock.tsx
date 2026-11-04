import { Check, Copy } from 'lucide-react'
import { useEffect, useState } from 'react'
import { toast } from '../stores/toast'

type Highlighter = Awaited<ReturnType<typeof import('shiki').createHighlighter>>

const SHIKI_LANGS = [
  'typescript',
  'javascript',
  'tsx',
  'rust',
  'python',
  'sql',
  'json',
  'markdown',
  'bash',
  'yaml',
]

let highlighterPromise: Promise<Highlighter> | null = null

function getHighlighter(): Promise<Highlighter> {
  highlighterPromise ??= import('shiki').then(({ createHighlighter }) =>
    createHighlighter({ themes: ['vesper'], langs: SHIKI_LANGS }),
  )
  return highlighterPromise
}

const ALIASES: Record<string, string> = {
  ts: 'typescript',
  js: 'javascript',
  py: 'python',
  sh: 'bash',
  zsh: 'bash',
  shell: 'bash',
  yml: 'yaml',
  md: 'markdown',
}

function shikiLang(language?: string | null): string {
  const raw = (language ?? '').toLowerCase()
  if (!raw || raw === 'text' || raw === 'plain') return 'text'
  const id = ALIASES[raw] ?? raw
  return SHIKI_LANGS.includes(id) ? id : 'text'
}

interface CodeBlockProps {
  code: string
  language?: string | null
  label?: string
}

export function CodeBlock({ code, language, label }: CodeBlockProps) {
  const [html, setHtml] = useState<string | null>(null)
  const [copied, setCopied] = useState(false)
  const lang = shikiLang(language)

  useEffect(() => {
    if (lang === 'text') {
      setHtml(null)
      return
    }
    let alive = true
    getHighlighter()
      .then((h) => {
        if (alive) setHtml(h.codeToHtml(code, { lang, theme: 'vesper' }))
      })
      .catch(() => {})
    return () => {
      alive = false
    }
  }, [code, lang])

  const copy = async () => {
    try {
      await navigator.clipboard.writeText(code)
      setCopied(true)
      toast('Copied to clipboard', 'success')
      window.setTimeout(() => setCopied(false), 1500)
    } catch {
      toast('Clipboard unavailable', 'error')
    }
  }

  return (
    <div className="overflow-hidden rounded-xl border border-line bg-codebg">
      <div className="flex items-center justify-between border-b border-line/70 px-3 py-1.5">
        <span className="font-mono text-[10px] tracking-widest text-faint uppercase">{label ?? lang}</span>
        <button
          type="button"
          onClick={copy}
          aria-label="Copy code"
          className="cursor-pointer rounded-md p-1 text-faint hover:text-ink"
        >
          {copied ? <Check size={14} className="text-accent" /> : <Copy size={14} />}
        </button>
      </div>
      {html ? (
        <div
          className="overflow-x-auto px-4 py-3 text-[13px] leading-relaxed"
          dangerouslySetInnerHTML={{ __html: html }}
        />
      ) : (
        <pre className="overflow-x-auto px-4 py-3 font-mono text-[13px] leading-relaxed text-ink/90">
          <code>{code}</code>
        </pre>
      )}
    </div>
  )
}
