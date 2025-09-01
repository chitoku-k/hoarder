import type { GraphQLError } from 'graphql'

export const MEDIUM_REPLICA_ENCODE_FAILED = 'MEDIUM_REPLICA_ENCODE_FAILED'

export interface MediumReplicaEncodeFailed extends GraphQLError {
  extensions: {
    details: {
      code: typeof MEDIUM_REPLICA_ENCODE_FAILED
    }
  }
}
