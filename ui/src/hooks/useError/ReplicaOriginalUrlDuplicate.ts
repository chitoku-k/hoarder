import type { GraphQLError } from 'graphql'

export const REPLICA_ORIGINAL_URL_DUPLICATE = 'REPLICA_ORIGINAL_URL_DUPLICATE'

export interface ReplicaOriginalUrlDuplicate extends GraphQLError {
  extensions: {
    details: {
      code: typeof REPLICA_ORIGINAL_URL_DUPLICATE
      data: {
        originalUrl: string
      }
    }
  }
}
