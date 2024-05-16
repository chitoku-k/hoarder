import type { GraphQLError } from 'graphql'

export const OBJECT_LIST_FAILED = 'OBJECT_LIST_FAILED'

export interface ObjectListFailed extends GraphQLError {
  extensions: {
    details: {
      code: typeof OBJECT_LIST_FAILED
      data: {
        url: string
      }
    }
  }
}
