import type { GraphQLError } from 'graphql'

export const MEDIUM_REPLICA_READ_FAILED = 'MEDIUM_REPLICA_READ_FAILED'

export interface MediumReplicaReadFailed extends GraphQLError {
  extensions: {
    details: {
      code: typeof MEDIUM_REPLICA_READ_FAILED
      data: {
      }
    }
  }
}
