import type { GraphQLError } from 'graphql'

export const TAG_CHILDREN_EXIST = 'TAG_CHILDREN_EXIST'

export interface TagChildrenExist extends GraphQLError {
  readonly extensions: {
    readonly details: {
      readonly code: typeof TAG_CHILDREN_EXIST
      readonly data: {
        readonly id: string
        readonly children: readonly string[]
      }
    }
  }
}
