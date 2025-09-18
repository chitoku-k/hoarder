import type { GraphQLError } from 'graphql'

export const MEDIUM_SOURCE_NOT_FOUND = 'MEDIUM_SOURCE_NOT_FOUND'

export interface MediumSourceNotFound extends GraphQLError {
  readonly extensions: {
    readonly details: {
      readonly code: typeof MEDIUM_SOURCE_NOT_FOUND
      readonly data: {
        readonly id: string
      }
    }
  }
}
