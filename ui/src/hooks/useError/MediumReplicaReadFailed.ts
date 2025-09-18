import type { GraphQLError } from 'graphql'

export const MEDIUM_REPLICA_READ_FAILED = 'MEDIUM_REPLICA_READ_FAILED'

export interface MediumReplicaReadFailed extends GraphQLError {
  readonly extensions: {
    readonly details: {
      readonly code: typeof MEDIUM_REPLICA_READ_FAILED
    }
  }
}
