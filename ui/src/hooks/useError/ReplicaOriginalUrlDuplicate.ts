import type { GraphQLError } from 'graphql'

export const REPLICA_ORIGINAL_URL_DUPLICATE = 'REPLICA_ORIGINAL_URL_DUPLICATE'

export interface ReplicaOriginalUrlDuplicate extends GraphQLError {
  readonly extensions: {
    readonly details: {
      readonly code: typeof REPLICA_ORIGINAL_URL_DUPLICATE
      readonly data: {
        readonly originalUrl: string
        readonly entry: {
          readonly name: string
          readonly url: string
          readonly kind: string
          readonly metadata: {
            readonly size: number
            readonly createdAt: string | null
            readonly updatedAt: string | null
            readonly accessedAt: string | null
          } | null
        } | null
      }
    }
  }
}
