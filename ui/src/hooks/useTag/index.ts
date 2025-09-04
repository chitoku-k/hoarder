import type { SkipToken } from '@apollo/client/react'
import { useSuspenseQuery } from '@apollo/client/react'

import type { TagQuery, TagQueryVariables } from '@/graphql/Tag'
import { TagDocument } from '@/graphql/Tag'

type Tag = TagQuery['tags'][number]

export function useTag(variables: TagQueryVariables | SkipToken): Tag | null {
  const options = typeof variables === 'symbol'
    ? variables
    : { variables } satisfies useSuspenseQuery.Options<TagQueryVariables>

  const { data } = useSuspenseQuery(TagDocument, options)
  if (!data) {
    return null
  }
  return data.tags[0] ?? null
}
