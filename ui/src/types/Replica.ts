export interface Replica {
  id: string
  displayOrder: number
  thumbnail?: {
    id: string
    width: number
    height: number
    url: string
    createdAt: string
    updatedAt: string
  } | null
  originalUrl: string
  url?: string | null
  mimeType?: string | null
  width?: number | null
  height?: number | null
  status: {
    phase: string
  }
  createdAt: string
  updatedAt: string
}
