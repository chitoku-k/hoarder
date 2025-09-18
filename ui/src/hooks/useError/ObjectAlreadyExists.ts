import type { GraphQLError } from 'graphql'

export const OBJECT_ALREADY_EXISTS = 'OBJECT_ALREADY_EXISTS'

export interface ObjectAlreadyExists extends GraphQLError {
  readonly extensions: {
    readonly details: {
      readonly code: typeof OBJECT_ALREADY_EXISTS
      readonly data: {
        readonly url: string
        readonly entry: {
          readonly name: string
          readonly url: string
          readonly kind: string
          readonly metadata: {
            readonly size: number
            readonly createdAt: string | null
            readonly updatedAt: string | null
            readonly accessedAt: string | null
          } | null
        } | null
      }
    }
  }
}
