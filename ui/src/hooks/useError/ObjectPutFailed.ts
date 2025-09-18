import type { GraphQLError } from 'graphql'

export const OBJECT_PUT_FAILED = 'OBJECT_PUT_FAILED'

export interface ObjectPutFailed extends GraphQLError {
  readonly extensions: {
    readonly details: {
      readonly code: typeof OBJECT_PUT_FAILED
      readonly data: {
        readonly url: string
      }
    }
  }
}
