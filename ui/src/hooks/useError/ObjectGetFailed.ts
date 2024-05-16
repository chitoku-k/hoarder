import type { GraphQLError } from 'graphql'

export const OBJECT_GET_FAILED = 'OBJECT_GET_FAILED'

export interface ObjectGetFailed extends GraphQLError {
  extensions: {
    details: {
      code: typeof OBJECT_GET_FAILED
      data: {
        url: string
      }
    }
  }
}
