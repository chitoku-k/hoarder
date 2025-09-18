import { useCallback } from 'react'
import { useSuspenseQuery } from '@apollo/client/react'

import type { AllMediaQuery } from '@/graphql/Media'
import { AllMediaDocument } from '@/graphql/Media'

type Media = AllMediaQuery['allMedia']['edges'][number]['node'][]

export function useMedia(number: number, options?: UseMediaOptions): [ Media, boolean, () => Promise<void> ] {
  const { data, fetchMore } = useSuspenseQuery(AllMediaDocument, {
    variables: {
      number,
      sourceIDs: options?.sourceIDs,
      tagTagTypeIDs: options?.tagTagTypeIDs?.map(({ tagID: tagId, typeID: tagTypeId }) => ({ tagId, tagTypeId })),
    },
  })

  const fetchNextPage = useCallback(async () => {
    await fetchMore({
      variables: {
        after: data.allMedia.pageInfo.endCursor,
      },
    })
  }, [ data, fetchMore ])

  return [
    data.allMedia.edges.map(({ node }) => node),
    data.allMedia.pageInfo.hasNextPage,
    fetchNextPage,
  ]
}

export interface UseMediaOptions {
  readonly sourceIDs?: readonly string[]
  readonly tagTagTypeIDs?: readonly {
    readonly tagID: string
    readonly typeID: string
  }[]
}
