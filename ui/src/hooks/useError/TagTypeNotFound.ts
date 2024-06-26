import type { GraphQLError } from 'graphql'

export const TAG_TYPE_NOT_FOUND = 'TAG_TYPE_NOT_FOUND'

export interface TagTypeNotFound extends GraphQLError {
  extensions: {
    details: {
      code: typeof TAG_TYPE_NOT_FOUND
      data: {
        id: string
      }
    }
  }
}
