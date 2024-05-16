import { useCallback } from 'react'
import { skipToken, useSuspenseQuery } from '@apollo/client'

import type { AllTagsQuery, AllTagsLikeQuery, TagsQuery, AllTagsLikeQueryVariables } from './documents.generated'
import { AllTagsDocument, AllTagsLikeDocument, TagsDocument } from './documents.generated'
export { AllTagsDocument, AllTagsLikeDocument, TagsDocument } from './documents.generated'

type RootTags = AllTagsQuery['allTags']['edges'][number]['node'][]
type TagsLike = AllTagsLikeQuery['allTagsLike']
type Tags = TagsQuery['tags'][number]['children']

export function useTags(parent: string): [ Tags, boolean, null ]

export function useTags(number: number): [ RootTags, boolean, () => Promise<void> ]

export function useTags(parentOrNumber: string | number): [ Tags, boolean, null ] | [ RootTags, boolean, () => Promise<void> ] {
  return typeof parentOrNumber === 'string'
    ? useTagsChildren(parentOrNumber)
    : useTagsRoot(parentOrNumber)
}

export function useTagsLike(variables: AllTagsLikeQueryVariables): TagsLike {
  const { data } = useSuspenseQuery(AllTagsLikeDocument, {
    variables,
    fetchPolicy: 'no-cache',
  })
  return data.allTagsLike
}

export function useTagsLikeSkip(): TagsLike {
  useSuspenseQuery(AllTagsLikeDocument, skipToken)
  return []
}

function useTagsChildren(id: string): [ Tags, boolean, null ] {
  const { data } = useSuspenseQuery(TagsDocument, {
    variables: {
      ids: [ id ],
    },
  })
  if (!data.tags[0]) {
    throw new Error('tag not found')
  }
  return [
    data.tags[0].children,
    false,
    null,
  ]
}

function useTagsRoot(number: number): [ RootTags, boolean, () => Promise<void> ] {
  const { data, fetchMore } = useSuspenseQuery(AllTagsDocument, {
    variables: {
      number,
    },
  })

  const fetchNextPage = useCallback(async () => {
    await fetchMore({
      variables: {
        after: data.allTags.pageInfo.endCursor,
      },
    })
  }, [ data, fetchMore ])

  return [
    data.allTags.edges.map(({ node }) => node),
    data.allTags.pageInfo.hasNextPage,
    fetchNextPage,
  ]
}
