import type { GraphQLError } from 'graphql'

export const OBJECT_NOT_FOUND = 'OBJECT_NOT_FOUND'

export interface ObjectNotFound extends GraphQLError {
  readonly extensions: {
    readonly details: {
      readonly code: typeof OBJECT_NOT_FOUND
      readonly data: {
        readonly url: string
      }
    }
  }
}
