import type { GraphQLError } from 'graphql'

export const SOURCE_METADATA_NOT_MATCH = 'SOURCE_METADATA_NOT_MATCH'

export interface SourceMetadataNotMatch extends GraphQLError {
  extensions: {
    details: {
      code: typeof SOURCE_METADATA_NOT_MATCH
      data: {
        slug: string
      }
    }
  }
}
