import type { GraphQLError } from 'graphql'

export const OBJECT_URL_UNSUPPORTED = 'OBJECT_URL_UNSUPPORTED'

export interface ObjectUrlUnsupported extends GraphQLError {
  extensions: {
    details: {
      code: typeof OBJECT_URL_UNSUPPORTED
      data: {
        url: string
      }
    }
  }
}
