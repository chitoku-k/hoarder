import { skipToken, useSuspenseQuery } from '@apollo/client'

import type { TagQuery, TagQueryVariables } from '@/graphql/Tag'
import { TagDocument } from '@/graphql/Tag'

type Tag = TagQuery['tags'][number]

export function useTag(variables: TagQueryVariables): Tag {
  const { data } = useSuspenseQuery(TagDocument, {
    variables,
  })
  if (!data.tags[0]) {
    throw new Error('tag not found')
  }
  return data.tags[0]
}

export function useTagSkip(): null {
  useSuspenseQuery(TagDocument, skipToken)
  return null
}
