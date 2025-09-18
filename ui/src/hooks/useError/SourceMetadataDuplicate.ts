import type { GraphQLError } from 'graphql'

export const SOURCE_METADATA_DUPLICATE = 'SOURCE_METADATA_DUPLICATE'

export interface SourceMetadataDuplicate extends GraphQLError {
  readonly extensions: {
    readonly details: {
      readonly code: typeof SOURCE_METADATA_DUPLICATE
      readonly data: {
        readonly id: string | null
      }
    }
  }
}
