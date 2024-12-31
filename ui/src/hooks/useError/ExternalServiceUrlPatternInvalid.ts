import type { GraphQLError } from 'graphql'

export const EXTERNAL_SERVICE_URL_PATTERN_INVALID = 'EXTERNAL_SERVICE_URL_PATTERN_INVALID'

export interface ExternalServiceUrlPatternInvalid extends GraphQLError {
  extensions: {
    details: {
      code: typeof EXTERNAL_SERVICE_URL_PATTERN_INVALID
      data: {
        urlPattern: string
        description: string | null
      }
    }
  }
}

