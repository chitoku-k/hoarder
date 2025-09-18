import type { GraphQLError } from 'graphql'

export const TAG_TYPE_NOT_FOUND = 'TAG_TYPE_NOT_FOUND'

export interface TagTypeNotFound extends GraphQLError {
  readonly extensions: {
    readonly details: {
      readonly code: typeof TAG_TYPE_NOT_FOUND
      readonly data: {
        readonly id: string
      }
    }
  }
}
