export interface ExternalService {
  id: string
  slug: string
  kind: string
  name: string
  baseUrl?: string | null
  urlPattern?: string | null
}
