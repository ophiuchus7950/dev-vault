import { javascript } from '@codemirror/lang-javascript'
import { json } from '@codemirror/lang-json'
import { markdown } from '@codemirror/lang-markdown'
import { python } from '@codemirror/lang-python'
import { rust } from '@codemirror/lang-rust'
import { sql } from '@codemirror/lang-sql'
import { Compartment, EditorState, Prec } from '@codemirror/state'
import { oneDark } from '@codemirror/theme-one-dark'
import { placeholder as cmPlaceholder, EditorView, keymap } from '@codemirror/view'
import { basicSetup } from 'codemirror'
import { useEffect, useRef } from 'react'

function langExtension(language?: string | null) {
  switch ((language ?? '').toLowerCase()) {
    case 'typescript':
      return javascript({ typescript: true, jsx: true })
    case 'javascript':
      return javascript({ jsx: true })
    case 'rust':
      return rust()
    case 'python':
      return python()
    case 'sql':
      return sql()
    case 'json':
      return json()
    case 'markdown':
      return markdown()
    default:
      return []
  }
}

interface CodeEditorProps {
  value: string
  onChange: (value: string) => void
  onSave: () => void
  language?: string | null
  placeholder?: string
}

export default function CodeEditor({ value, onChange, onSave, language, placeholder }: CodeEditorProps) {
  const hostRef = useRef<HTMLDivElement>(null)
  const viewRef = useRef<EditorView | null>(null)
  const langCompartment = useRef(new Compartment()).current
  const onChangeRef = useRef(onChange)
  const onSaveRef = useRef(onSave)
  onChangeRef.current = onChange
  onSaveRef.current = onSave

  // biome-ignore lint/correctness/useExhaustiveDependencies: `value` intentionally omitted — synced in a separate effect
  useEffect(() => {
    const host = hostRef.current
    if (!host) return
    const view = new EditorView({
      parent: host,
      state: EditorState.create({
        doc: value,
        extensions: [
          basicSetup,
          oneDark,
          EditorView.lineWrapping,
          placeholder ? cmPlaceholder(placeholder) : [],
          langCompartment.of(langExtension(language)),
          EditorView.theme({
            '&': { fontSize: '13px', backgroundColor: 'transparent', height: '100%' },
            '.cm-scroller': {
              fontFamily: 'var(--font-mono)',
              lineHeight: '1.6',
            },
            '.cm-gutters': { backgroundColor: 'transparent', border: 'none' },
          }),
          Prec.highest(
            keymap.of([
              {
                key: 'Mod-s',
                preventDefault: true,
                run: () => {
                  onSaveRef.current()
                  return true
                },
              },
            ]),
          ),
          EditorView.updateListener.of((update) => {
            if (update.docChanged) onChangeRef.current(update.state.doc.toString())
          }),
        ],
      }),
    })
    viewRef.current = view
    return () => {
      view.destroy()
      viewRef.current = null
    }
  }, [language, langCompartment])

  useEffect(() => {
    const view = viewRef.current
    if (!view) return
    const current = view.state.doc.toString()
    if (current !== value) {
      view.dispatch({ changes: { from: 0, to: current.length, insert: value } })
    }
  }, [value])

  return (
    <div
      ref={hostRef}
      className="min-h-[240px] overflow-hidden rounded-xl border border-line bg-codebg [&_.cm-editor]:max-h-[420px]"
    />
  )
}
