import type { GraphQLError } from 'graphql'

export const OBJECT_DELETE_FAILED = 'OBJECT_DELETE_FAILED'

export interface ObjectDeleteFailed extends GraphQLError {
  extensions: {
    details: {
      code: typeof OBJECT_DELETE_FAILED
      data: {
        url: string
      }
    }
  }
}
