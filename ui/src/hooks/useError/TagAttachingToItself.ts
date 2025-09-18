import type { GraphQLError } from 'graphql'

export const TAG_ATTACHING_TO_ITSELF = 'TAG_ATTACHING_TO_ITSELF'

export interface TagAttachingToItself extends GraphQLError {
  readonly extensions: {
    readonly details: {
      readonly code: typeof TAG_ATTACHING_TO_ITSELF
      readonly data: {
        readonly id: string
      }
    }
  }
}
