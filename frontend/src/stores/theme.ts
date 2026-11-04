import { create } from 'zustand'

export type ThemePref = 'light' | 'dark' | 'system'

const STORAGE_KEY = 'devvault-theme'

function resolve(pref: ThemePref): 'dark' | 'light' {
  if (pref === 'system') {
    return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light'
  }
  return pref
}

export function applyTheme(pref: ThemePref): void {
  document.documentElement.classList.toggle('light', resolve(pref) === 'light')
}

function storedPref(): ThemePref {
  try {
    const value = localStorage.getItem(STORAGE_KEY)
    if (value === 'light' || value === 'dark' || value === 'system') return value
  } catch {
    // storage unavailable
  }
  return 'dark'
}

interface ThemeState {
  pref: ThemePref
  resolved: 'dark' | 'light'
  setPref: (pref: ThemePref) => void
  cycle: () => void
}

export const useTheme = create<ThemeState>((set, get) => ({
  pref: storedPref(),
  resolved: resolve(storedPref()),
  setPref: (pref) => {
    try {
      localStorage.setItem(STORAGE_KEY, pref)
    } catch {
      // storage unavailable
    }
    applyTheme(pref)
    set({ pref, resolved: resolve(pref) })
  },
  cycle: () => {
    const order: ThemePref[] = ['dark', 'light', 'system']
    const next = order[(order.indexOf(get().pref) + 1) % order.length]
    get().setPref(next)
  },
}))

export function watchSystemTheme(): void {
  const mq = window.matchMedia('(prefers-color-scheme: dark)')
  mq.addEventListener('change', () => applyTheme(useTheme.getState().pref))
}
