import type { GraphQLError } from 'graphql'

export const REPLICA_NOT_FOUND = 'REPLICA_NOT_FOUND'

export interface ReplicaNotFound extends GraphQLError {
  readonly extensions: {
    readonly details: {
      readonly code: typeof REPLICA_NOT_FOUND
      readonly data: {
        readonly id: string
      }
    }
  }
}
