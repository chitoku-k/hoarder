import { useCallback } from 'react'
import { useSuspenseQuery } from '@apollo/client'

import type { AllMediaQuery } from '@/graphql/Media'
import { AllMediaDocument } from '@/graphql/Media'

type Media = AllMediaQuery['allMedia']['edges'][number]['node'][]

export function useMedia(number: number): [ Media, boolean, () => Promise<void> ] {
  const { data, fetchMore } = useSuspenseQuery(AllMediaDocument, {
    variables: {
      number,
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

