import type { GraphQLError } from 'graphql'

export const MEDIUM_REPLICAS_NOT_MATCH = 'MEDIUM_REPLICAS_NOT_MATCH'

export interface MediumReplicasNotMatch extends GraphQLError {
  readonly extensions: {
    readonly details: {
      readonly code: typeof MEDIUM_REPLICAS_NOT_MATCH
      readonly data: {
        readonly mediumId: string
        readonly expectedReplicas: readonly string[]
        readonly actualReplicas: readonly string[]
      }
    }
  }
}
