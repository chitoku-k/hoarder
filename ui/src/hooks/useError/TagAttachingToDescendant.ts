import type { GraphQLError } from 'graphql'

export const TAG_ATTACHING_TO_DESCENDANT = 'TAG_ATTACHING_TO_DESCENDANT'

export interface TagAttachingToDescendant extends GraphQLError {
  extensions: {
    details: {
      code: typeof TAG_ATTACHING_TO_DESCENDANT
      data: {
        id: string
      }
    }
  }
}
