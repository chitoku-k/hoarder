import type { GraphQLError } from 'graphql'

export const THUMBNAIL_NOT_FOUND = 'THUMBNAIL_NOT_FOUND'

export interface ThumbnailNotFound extends GraphQLError {
  extensions: {
    details: {
      code: typeof THUMBNAIL_NOT_FOUND
      data: {
        id: string
      }
    }
  }
}
