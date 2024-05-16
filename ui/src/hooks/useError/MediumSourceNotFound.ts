import type { GraphQLError } from 'graphql'

export const MEDIUM_SOURCE_NOT_FOUND = 'MEDIUM_SOURCE_NOT_FOUND'

export interface MediumSourceNotFound extends GraphQLError {
  extensions: {
    details: {
      code: typeof MEDIUM_SOURCE_NOT_FOUND
      data: {
        id: string
      }
    }
  }
}
