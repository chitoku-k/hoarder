export interface ExternalService {
  readonly id: string
  readonly slug: string
  readonly kind: string
  readonly name: string
  readonly baseUrl?: string | null
  readonly urlPattern?: string | null
}
