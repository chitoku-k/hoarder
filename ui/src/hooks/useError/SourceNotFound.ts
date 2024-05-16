import type { GraphQLError } from 'graphql'

export const SOURCE_NOT_FOUND = 'SOURCE_NOT_FOUND'

export interface SourceNotFound extends GraphQLError {
  extensions: {
    details: {
      code: typeof SOURCE_NOT_FOUND
      data: {
        id: string
      }
    }
  }
}
