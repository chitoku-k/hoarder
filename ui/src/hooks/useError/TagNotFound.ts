import type { GraphQLError } from 'graphql'

export const TAG_NOT_FOUND = 'TAG_NOT_FOUND'

export interface TagNotFound extends GraphQLError {
  readonly extensions: {
    readonly details: {
      readonly code: typeof TAG_NOT_FOUND
      readonly data: {
        readonly id: string
      }
    }
  }
}
