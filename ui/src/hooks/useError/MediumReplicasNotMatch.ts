import type { GraphQLError } from 'graphql'

export const MEDIUM_REPLICAS_NOT_MATCH = 'MEDIUM_REPLICAS_NOT_MATCH'

export interface MediumReplicasNotMatch extends GraphQLError {
  extensions: {
    details: {
      code: typeof MEDIUM_REPLICAS_NOT_MATCH
      data: {
        mediumId: string
        expectedReplicas: string[]
        actualReplicas: string[]
      }
    }
  }
}
