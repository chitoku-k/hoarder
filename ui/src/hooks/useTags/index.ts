import { useCallback } from 'react'
import type { SkipToken } from '@apollo/client/react'
import { useSuspenseQuery } from '@apollo/client/react'

import type { AllTagsLikeQuery, AllTagsLikeQueryVariables, AllTagsQuery, AllTagsQueryVariables, TagsQuery, TagsQueryVariables } from '@/graphql/Tags'
import { AllTagsDocument, AllTagsLikeDocument, TagsDocument } from '@/graphql/Tags'

type RootTags = AllTagsQuery['allTags']['edges'][number]['node'][]
type TagsLike = AllTagsLikeQuery['allTagsLike']
type Tags = TagsQuery['tags'][number]['children']

export function useTagsChildren(variables: TagsQueryVariables | SkipToken): Tags | null {
  const options = typeof variables === 'symbol'
    ? variables
    : { variables } satisfies useSuspenseQuery.Options<TagsQueryVariables>

  const { data } = useSuspenseQuery(TagsDocument, options)
  if (!data) {
    return null
  }
  if (!data.tags[0]) {
    throw new Error('tag not found')
  }
  return data.tags[0].children
}

export function useTagsRoot(variables: AllTagsQueryVariables | SkipToken): [ RootTags, boolean, () => Promise<void> ] | null {
  const options = typeof variables === 'symbol'
    ? variables
    : { variables } satisfies useSuspenseQuery.Options<AllTagsQueryVariables>

  const { data, fetchMore } = useSuspenseQuery(AllTagsDocument, options)

  const fetchNextPage = useCallback(async () => {
    if (!data) {
      throw new Error('unreachable')
    }
    await fetchMore({
      variables: {
        after: data.allTags.pageInfo.endCursor,
      },
    })
  }, [ data, fetchMore ])

  if (!data) {
    return null
  }

  return [
    data.allTags.edges.map(({ node }) => node),
    data.allTags.pageInfo.hasNextPage,
    fetchNextPage,
  ]
}

export function useTagsLike(variables: AllTagsLikeQueryVariables | SkipToken): TagsLike | null {
  const options = typeof variables === 'symbol'
    ? variables
    : { variables, fetchPolicy: 'no-cache' } satisfies useSuspenseQuery.Options<AllTagsLikeQueryVariables>

  const { data } = useSuspenseQuery(AllTagsLikeDocument, options)
  return data?.allTagsLike ?? null
}
