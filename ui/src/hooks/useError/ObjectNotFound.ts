import type { GraphQLError } from 'graphql'

export const OBJECT_NOT_FOUND = 'OBJECT_NOT_FOUND'

export interface ObjectNotFound extends GraphQLError {
  extensions: {
    details: {
      code: typeof OBJECT_NOT_FOUND
      data: {
        url: string
      }
    }
  }
}
