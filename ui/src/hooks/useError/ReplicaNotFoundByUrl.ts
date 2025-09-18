import type { GraphQLError } from 'graphql'

export const REPLICA_NOT_FOUND_BY_URL = 'REPLICA_NOT_FOUND_BY_URL'

export interface ReplicaNotFoundByUrl extends GraphQLError {
  readonly extensions: {
    readonly details: {
      readonly code: typeof REPLICA_NOT_FOUND_BY_URL
      readonly data: {
        readonly originalUrl: string
      }
    }
  }
}
