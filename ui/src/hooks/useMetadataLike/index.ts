import { skipToken, useSuspenseQuery } from '@apollo/client/react'

import type { MetadataLikeQuery } from '@/graphql/MetadataLike'
import { MetadataLikeDocument } from '@/graphql/MetadataLike'

export interface MetadataLike {
  sources: {
    id: MetadataLikeQuery['allSourcesLikeId']
    url: MetadataLikeQuery['allSourcesLikeUrl']
  }
  tags: MetadataLikeQuery['allTagsLike']
}

export function useMetadataLike(like: string): MetadataLike {
  const { data } = useSuspenseQuery(MetadataLikeDocument, {
    variables: {
      like,
    },
  })
  return {
    sources: {
      id: data.allSourcesLikeId,
      url: data.allSourcesLikeUrl,
    },
    tags: data.allTagsLike,
  }
}

export function useMetadataLikeSkip(): MetadataLike {
  useSuspenseQuery(MetadataLikeDocument, skipToken)
  return {
    sources: {
      id: [],
      url: [],
    },
    tags: [],
  }
}
