import type { GraphQLError } from 'graphql'

export const OBJECT_DELETE_FAILED = 'OBJECT_DELETE_FAILED'

export interface ObjectDeleteFailed extends GraphQLError {
  readonly extensions: {
    readonly details: {
      readonly code: typeof OBJECT_DELETE_FAILED
      readonly data: {
        readonly url: string
      }
    }
  }
}
