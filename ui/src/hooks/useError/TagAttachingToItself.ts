import type { GraphQLError } from 'graphql'

export const TAG_ATTACHING_TO_ITSELF = 'TAG_ATTACHING_TO_ITSELF'

export interface TagAttachingToItself extends GraphQLError {
  extensions: {
    details: {
      code: typeof TAG_ATTACHING_TO_ITSELF
      data: {
        id: string
      }
    }
  }
}
