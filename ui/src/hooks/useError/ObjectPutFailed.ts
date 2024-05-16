import type { GraphQLError } from 'graphql'

export const OBJECT_PUT_FAILED = 'OBJECT_PUT_FAILED'

export interface ObjectPutFailed extends GraphQLError {
  extensions: {
    details: {
      code: typeof OBJECT_PUT_FAILED
      data: {
        url: string
      }
    }
  }
}
