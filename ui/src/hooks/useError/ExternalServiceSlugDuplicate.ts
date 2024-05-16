import type { GraphQLError } from 'graphql'

export const EXTERNAL_SERVICE_SLUG_DUPLICATE = 'EXTERNAL_SERVICE_SLUG_DUPLICATE'

export interface ExternalServiceSlugDuplicate extends GraphQLError {
  extensions: {
    details: {
      code: typeof EXTERNAL_SERVICE_SLUG_DUPLICATE
      data: {
        slug: string
      }
    }
  }
}
