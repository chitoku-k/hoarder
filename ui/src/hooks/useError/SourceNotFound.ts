import type { GraphQLError } from 'graphql'

export const SOURCE_NOT_FOUND = 'SOURCE_NOT_FOUND'

export interface SourceNotFound extends GraphQLError {
  readonly extensions: {
    readonly details: {
      readonly code: typeof SOURCE_NOT_FOUND
      readonly data: {
        readonly id: string
      }
    }
  }
}
