import type { GraphQLError } from 'graphql'

export const THUMBNAIL_NOT_FOUND = 'THUMBNAIL_NOT_FOUND'

export interface ThumbnailNotFound extends GraphQLError {
  readonly extensions: {
    readonly details: {
      readonly code: typeof THUMBNAIL_NOT_FOUND
      readonly data: {
        readonly id: string
      }
    }
  }
}
