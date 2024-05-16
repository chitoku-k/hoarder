import type { GraphQLError } from 'graphql'

export const MEDIUM_NOT_FOUND = 'MEDIUM_NOT_FOUND'

export interface MediumNotFound extends GraphQLError {
  extensions: {
    details: {
      code: typeof MEDIUM_NOT_FOUND
      data: {
        id: string
      }
    }
  }
}
