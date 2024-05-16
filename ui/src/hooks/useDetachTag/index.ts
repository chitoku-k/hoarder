import { useCallback } from 'react'
import type { ApolloError } from '@apollo/client'
import { useMutation } from '@apollo/client'

import { AllTagsDocument, TagDocument, TagsDocument } from '@/hooks'

import type { DetachTagMutation, DetachTagMutationVariables } from './documents.generated'
import { DetachTagDocument } from './documents.generated'
export { DetachTagDocument } from './documents.generated'

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
