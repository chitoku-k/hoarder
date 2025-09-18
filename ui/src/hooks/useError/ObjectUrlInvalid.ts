import type { GraphQLError } from 'graphql'

export const OBJECT_URL_INVALID = 'OBJECT_URL_INVALID'

export interface ObjectUrlInvalid extends GraphQLError {
  readonly extensions: {
    readonly details: {
      readonly code: typeof OBJECT_URL_INVALID
      readonly data: {
        readonly url: string
      }
    }
  }
}
