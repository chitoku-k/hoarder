import type { GraphQLError } from 'graphql'

export const MEDIUM_TAG_NOT_FOUND = 'MEDIUM_TAG_NOT_FOUND'

export interface MediumTagNotFound extends GraphQLError {
  extensions: {
    details: {
      code: typeof MEDIUM_TAG_NOT_FOUND
      data: {
        id: string
      }
    }
  }
}
