import { useCallback } from 'react'
import type { ApolloError } from '@apollo/client'
import { useMutation } from '@apollo/client'

import type { DetachTagMutation, DetachTagMutationVariables } from '@/graphql/DetachTag'
import { DetachTagDocument } from '@/graphql/DetachTag'
import { TagDocument } from '@/graphql/Tag'
import { AllTagsDocument, TagsDocument } from '@/graphql/Tags'

type DetachTag = DetachTagMutation['detachTag']

export function useDetachTag(): [
  (variables: DetachTagMutationVariables) => Promise<DetachTag>,
  { data?: DetachTag, loading: boolean, error?: ApolloError },
] {
  const [ detachTag, { data, loading, error } ] = useMutation(DetachTagDocument)
  return [
    useCallback(async (variables: DetachTagMutationVariables) => {
      const { data } = await detachTag({
        variables,
        awaitRefetchQueries: true,
        refetchQueries: [
          AllTagsDocument,
          TagDocument,
          TagsDocument,
        ],
      })
      return data?.detachTag!
    }, [ detachTag ]),
    {
      data: data?.detachTag,
      loading,
      error,
    },
  ]
}
