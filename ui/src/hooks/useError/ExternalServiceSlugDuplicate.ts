import type { GraphQLError } from 'graphql'

export const EXTERNAL_SERVICE_SLUG_DUPLICATE = 'EXTERNAL_SERVICE_SLUG_DUPLICATE'

export interface ExternalServiceSlugDuplicate extends GraphQLError {
  readonly extensions: {
    readonly details: {
      readonly code: typeof EXTERNAL_SERVICE_SLUG_DUPLICATE
      readonly data: {
        readonly slug: string
      }
    }
  }
}
