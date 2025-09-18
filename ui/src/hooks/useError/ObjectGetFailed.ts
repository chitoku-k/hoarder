import type { GraphQLError } from 'graphql'

export const OBJECT_GET_FAILED = 'OBJECT_GET_FAILED'

export interface ObjectGetFailed extends GraphQLError {
  readonly extensions: {
    readonly details: {
      readonly code: typeof OBJECT_GET_FAILED
      readonly data: {
        readonly url: string
      }
    }
  }
}
