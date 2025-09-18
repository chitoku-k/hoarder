import type { GraphQLError } from 'graphql'

export const EXTERNAL_SERVICE_URL_PATTERN_INVALID = 'EXTERNAL_SERVICE_URL_PATTERN_INVALID'

export interface ExternalServiceUrlPatternInvalid extends GraphQLError {
  readonly extensions: {
    readonly details: {
      readonly code: typeof EXTERNAL_SERVICE_URL_PATTERN_INVALID
      readonly data: {
        readonly urlPattern: string
        readonly description: string | null
      }
    }
  }
}
