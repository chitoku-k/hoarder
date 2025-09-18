import type { GraphQLError } from 'graphql'

export const OBJECT_LIST_FAILED = 'OBJECT_LIST_FAILED'

export interface ObjectListFailed extends GraphQLError {
  readonly extensions: {
    readonly details: {
      readonly code: typeof OBJECT_LIST_FAILED
      readonly data: {
        readonly url: string
      }
    }
  }
}
