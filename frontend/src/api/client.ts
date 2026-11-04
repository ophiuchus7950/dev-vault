import type { ErrorDetail } from './types'

export const API_BASE = '/api/v1'

export class ApiError extends Error {
  readonly code: string
  readonly status: number
  readonly details: ErrorDetail[]

  constructor(message: string, code: string, status: number, details: ErrorDetail[]) {
    super(message)
    this.name = 'ApiError'
    this.code = code
    this.status = status
    this.details = details
  }

  fieldErrors(): Record<string, string> {
    return Object.fromEntries(this.details.map((d) => [d.field, d.message]))
  }
}

export type QueryValue = string | number | boolean | undefined | null | (string | number)[]

export interface HttpOptions {
  method?: 'GET' | 'POST' | 'PATCH' | 'PUT' | 'DELETE'
  body?: unknown
  ifMatch?: string
  query?: Record<string, QueryValue>
}

export interface HttpResult<T> {
  data: T
  etag: string | null
}

export async function http<T>(path: string, opts: HttpOptions = {}): Promise<HttpResult<T>> {
  const url = new URL(API_BASE + path, window.location.origin)
  for (const [key, value] of Object.entries(opts.query ?? {})) {
    if (value === undefined || value === null || value === '' || value === false) continue
    if (Array.isArray(value)) {
      for (const item of value) url.searchParams.append(key, String(item))
    } else {
      url.searchParams.append(key, String(value))
    }
  }

  const headers: Record<string, string> = {}
  if (opts.body !== undefined) headers['content-type'] = 'application/json'
  if (opts.ifMatch) headers['if-match'] = opts.ifMatch

  const res = await fetch(url, {
    method: opts.method ?? 'GET',
    headers,
    body: opts.body === undefined ? undefined : JSON.stringify(opts.body),
  })

  if (!res.ok) {
    let code = 'HTTP_ERROR'
    let message = res.statusText || `Request failed (${res.status})`
    let details: ErrorDetail[] = []
    try {
      const payload = (await res.json()) as {
        error?: { code?: string; message?: string; details?: ErrorDetail[] }
      }
      if (payload?.error) {
        code = payload.error.code ?? code
        message = payload.error.message ?? message
        details = payload.error.details ?? []
      }
    } catch {
      // non-JSON error body; keep defaults
    }
    throw new ApiError(message, code, res.status, details)
  }

  const etag = res.headers.get('etag')
  if (res.status === 204) return { data: undefined as T, etag }
  return { data: (await res.json()) as T, etag }
}
