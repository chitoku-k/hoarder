import type { GraphQLError } from 'graphql'

export const OBJECT_PATH_INVALID = 'OBJECT_PATH_INVALID'

export interface ObjectPathInvalid extends GraphQLError {
  extensions: {
    details: {
      code: typeof OBJECT_PATH_INVALID
    }
  }
}
