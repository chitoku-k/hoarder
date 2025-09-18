import type { GraphQLError } from 'graphql'

export const SOURCE_METADATA_NOT_MATCH = 'SOURCE_METADATA_NOT_MATCH'

export interface SourceMetadataNotMatch extends GraphQLError {
  readonly extensions: {
    readonly details: {
      readonly code: typeof SOURCE_METADATA_NOT_MATCH
      readonly data: {
        readonly slug: string
      }
    }
  }
}
