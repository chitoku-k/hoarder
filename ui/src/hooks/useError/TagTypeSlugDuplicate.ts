import type { GraphQLError } from 'graphql'

export const TAG_TYPE_SLUG_DUPLICATE = 'TAG_TYPE_SLUG_DUPLICATE'

export interface TagTypeSlugDuplicate extends GraphQLError {
  extensions: {
    details: {
      code: typeof TAG_TYPE_SLUG_DUPLICATE
      data: {
        slug: string
      }
    }
  }
}
