import type { GraphQLError } from 'graphql'

export const OBJECT_ALREADY_EXISTS = 'OBJECT_ALREADY_EXISTS'

export interface ObjectAlreadyExists extends GraphQLError {
  extensions: {
    details: {
      code: typeof OBJECT_ALREADY_EXISTS
      data: {
        url: string
        entry: {
          name: string
          url: string
          kind: string
          metadata: {
            size: number
            createdAt: string
            updatedAt: string
            accessedAt: string
          }
        } | null
      }
    }
  }
}
