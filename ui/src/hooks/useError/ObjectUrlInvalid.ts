import type { GraphQLError } from 'graphql'

export const OBJECT_URL_INVALID = 'OBJECT_URL_INVALID'

export interface ObjectUrlInvalid extends GraphQLError {
  extensions: {
    details: {
      code: typeof OBJECT_URL_INVALID
      data: {
        url: string
      }
    }
  }
}
