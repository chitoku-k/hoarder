import type { GraphQLError } from 'graphql'

export const MEDIUM_NOT_FOUND = 'MEDIUM_NOT_FOUND'

export interface MediumNotFound extends GraphQLError {
  readonly extensions: {
    readonly details: {
      readonly code: typeof MEDIUM_NOT_FOUND
      readonly data: {
        readonly id: string
      }
    }
  }
}
