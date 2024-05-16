import type { GraphQLError } from 'graphql'

export const REPLICA_NOT_FOUND_BY_URL = 'REPLICA_NOT_FOUND_BY_URL'

export interface ReplicaNotFoundByUrl extends GraphQLError {
  extensions: {
    details: {
      code: typeof REPLICA_NOT_FOUND_BY_URL
      data: {
        originalUrl: string
      }
    }
  }
}
