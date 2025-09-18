import type { GraphQLError } from 'graphql'

export const TAG_TYPE_SLUG_DUPLICATE = 'TAG_TYPE_SLUG_DUPLICATE'

export interface TagTypeSlugDuplicate extends GraphQLError {
  readonly extensions: {
    readonly details: {
      readonly code: typeof TAG_TYPE_SLUG_DUPLICATE
      readonly data: {
        readonly slug: string
      }
    }
  }
}
