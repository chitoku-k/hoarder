import type { GraphQLError } from 'graphql'

export const TAG_CHILDREN_EXIST = 'TAG_CHILDREN_EXIST'

export interface TagChildrenExist extends GraphQLError {
  extensions: {
    details: {
      code: typeof TAG_CHILDREN_EXIST
      data: {
        id: string
        children: string[]
      }
    }
  }
}
