import type { GraphQLError } from 'graphql'

export const MEDIUM_REPLICA_UNSUPPORTED = 'MEDIUM_REPLICA_UNSUPPORTED'

export interface MediumReplicaUnsupported extends GraphQLError {
  extensions: {
    details: {
      code: typeof MEDIUM_REPLICA_UNSUPPORTED
    }
  }
}
