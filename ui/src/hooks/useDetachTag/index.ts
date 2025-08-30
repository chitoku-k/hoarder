import { useCallback } from 'react'
import type { ErrorLike } from '@apollo/client'
import { useMutation } from '@apollo/client/react'

import type { DetachTagMutation, DetachTagMutationVariables } from '@/graphql/DetachTag'
import { DetachTagDocument } from '@/graphql/DetachTag'
import { AllTagsDocument, TagsDocument } from '@/graphql/Tags'

type DetachTag = DetachTagMutation['detachTag']

export function useDetachTag(): [
  (variables: DetachTagMutationVariables) => Promise<DetachTag>,
  { data?: DetachTag, loading: boolean, error?: ErrorLike },
] {
  const [ detachTag, { data, loading, error } ] = useMutation(DetachTagDocument)
  return [
    useCallback(async (variables: DetachTagMutationVariables) => {
      const { data, error } = await detachTag({
        variables,
        awaitRefetchQueries: true,
        refetchQueries: [
          AllTagsDocument,
          TagsDocument,
        ],
      })
      if (!data) {
        throw error
      }
      return data.detachTag
    }, [ detachTag ]),
    {
      data: data?.detachTag,
      loading,
      error,
    },
  ]
}
