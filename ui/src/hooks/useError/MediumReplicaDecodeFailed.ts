import type { GraphQLError } from 'graphql'

export const MEDIUM_REPLICA_DECODE_FAILED = 'MEDIUM_REPLICA_DECODE_FAILED'

export interface MediumReplicaDecodeFailed extends GraphQLError {
  extensions: {
    details: {
      code: typeof MEDIUM_REPLICA_DECODE_FAILED
      data: {
      }
    }
  }
}
