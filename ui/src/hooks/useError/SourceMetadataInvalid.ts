import type { GraphQLError } from 'graphql'

export const SOURCE_METADATA_INVALID = 'SOURCE_METADATA_INVALID'

export interface SourceMetadataInvalid extends GraphQLError {
  extensions: {
    details: {
      code: typeof SOURCE_METADATA_INVALID
    }
  }
}
