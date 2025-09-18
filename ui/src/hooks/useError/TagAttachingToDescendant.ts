import type { GraphQLError } from 'graphql'

export const TAG_ATTACHING_TO_DESCENDANT = 'TAG_ATTACHING_TO_DESCENDANT'

export interface TagAttachingToDescendant extends GraphQLError {
  readonly extensions: {
    readonly details: {
      readonly code: typeof TAG_ATTACHING_TO_DESCENDANT
      readonly data: {
        readonly id: string
      }
    }
  }
}
