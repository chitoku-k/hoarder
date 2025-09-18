export interface Replica {
  readonly id: string
  readonly displayOrder: number
  readonly thumbnail?: {
    readonly id: string
    readonly width: number
    readonly height: number
    readonly url: string
    readonly createdAt: string
    readonly updatedAt: string
  } | null
  readonly originalUrl: string
  readonly url?: string | null
  readonly mimeType?: string | null
  readonly width?: number | null
  readonly height?: number | null
  readonly status: {
    readonly phase: string
  }
  readonly createdAt: string
  readonly updatedAt: string
}
