import type { GraphQLError } from 'graphql'

export const EXTERNAL_SERVICE_NOT_FOUND = 'EXTERNAL_SERVICE_NOT_FOUND'

export interface ExternalServiceNotFound extends GraphQLError {
  readonly extensions: {
    readonly details: {
      readonly code: typeof EXTERNAL_SERVICE_NOT_FOUND
      readonly data: {
        readonly id: string
      }
    }
  }
}
