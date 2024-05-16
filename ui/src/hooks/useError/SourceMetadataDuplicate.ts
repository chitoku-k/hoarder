import type { GraphQLError } from 'graphql'

export const SOURCE_METADATA_DUPLICATE = 'SOURCE_METADATA_DUPLICATE'

export interface SourceMetadataDuplicate extends GraphQLError {
  extensions: {
    details: {
      code: typeof SOURCE_METADATA_DUPLICATE
      data: {
        id: string | null
      }
    }
  }
}
