import type { GraphQLError } from 'graphql'

export const OBJECT_URL_UNSUPPORTED = 'OBJECT_URL_UNSUPPORTED'

export interface ObjectUrlUnsupported extends GraphQLError {
  readonly extensions: {
    readonly details: {
      readonly code: typeof OBJECT_URL_UNSUPPORTED
      readonly data: {
        readonly url: string
      }
    }
  }
}
