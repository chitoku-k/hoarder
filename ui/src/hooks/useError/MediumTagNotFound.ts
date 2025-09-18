import type { GraphQLError } from 'graphql'

export const MEDIUM_TAG_NOT_FOUND = 'MEDIUM_TAG_NOT_FOUND'

export interface MediumTagNotFound extends GraphQLError {
  readonly extensions: {
    readonly details: {
      readonly code: typeof MEDIUM_TAG_NOT_FOUND
      readonly data: {
        readonly id: string
      }
    }
  }
}
