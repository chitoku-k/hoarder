import type { GraphQLError } from 'graphql'

export const REPLICA_NOT_FOUND = 'REPLICA_NOT_FOUND'

export interface ReplicaNotFound extends GraphQLError {
  extensions: {
    details: {
      code: typeof REPLICA_NOT_FOUND
      data: {
        id: string
      }
    }
  }
}
