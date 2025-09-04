import type { SkipToken } from '@apollo/client/react'
import { useSuspenseQuery } from '@apollo/client/react'

import type { MetadataLikeQuery, MetadataLikeQueryVariables } from '@/graphql/MetadataLike'
import { MetadataLikeDocument } from '@/graphql/MetadataLike'

export interface MetadataLike {
  sources: {
    id: MetadataLikeQuery['allSourcesLikeId']
    url: MetadataLikeQuery['allSourcesLikeUrl']
  }
  tags: MetadataLikeQuery['allTagsLike']
}

export function useMetadataLike(variables: MetadataLikeQueryVariables | SkipToken): Partial<MetadataLike> {
  const options = typeof variables === 'symbol'
    ? variables
    : { variables } satisfies useSuspenseQuery.Options<MetadataLikeQueryVariables>

  const { data } = useSuspenseQuery(MetadataLikeDocument, options)
  if (!data) {
    return {}
  }
  return {
    sources: {
      id: data.allSourcesLikeId,
      url: data.allSourcesLikeUrl,
    },
    tags: data.allTagsLike,
  }
}
