import type { GraphQLError } from 'graphql'

export const EXTERNAL_SERVICE_NOT_FOUND = 'EXTERNAL_SERVICE_NOT_FOUND'

export interface ExternalServiceNotFound extends GraphQLError {
  extensions: {
    details: {
      code: typeof EXTERNAL_SERVICE_NOT_FOUND
      data: {
        id: string
      }
    }
  }
}
