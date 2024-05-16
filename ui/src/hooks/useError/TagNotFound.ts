import type { GraphQLError } from 'graphql'

export const TAG_NOT_FOUND = 'TAG_NOT_FOUND'

export interface TagNotFound extends GraphQLError {
  extensions: {
    details: {
      code: typeof TAG_NOT_FOUND
      data: {
        id: string
      }
    }
  }
}
