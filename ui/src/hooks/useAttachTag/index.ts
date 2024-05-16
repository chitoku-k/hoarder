import { useCallback } from 'react'
import type { ApolloError } from '@apollo/client'
import { useMutation } from '@apollo/client'

import { AllTagsDocument, TagDocument, TagsDocument } from '@/hooks'

import type { AttachTagMutation, AttachTagMutationVariables } from './documents.generated'
import { AttachTagDocument } from './documents.generated'
export { AttachTagDocument } from './documents.generated'

type AttachTag = AttachTagMutation['attachTag']

export function useAttachTag(): [
  (variables: AttachTagMutationVariables) => Promise<AttachTag>,
  { data?: AttachTag, loading: boolean, error?: ApolloError },
] {
  const [ attachTag, { data, loading, error } ] = useMutation(AttachTagDocument)
  return [
    useCallback(async (variables: AttachTagMutationVariables) => {
      const { data } = await attachTag({
        variables,
        awaitRefetchQueries: true,
        refetchQueries: [
          AllTagsDocument,
          TagDocument,
          TagsDocument,
        ],
      })
      return data?.attachTag!
    }, [ attachTag ]),
    {
      data: data?.attachTag,
      loading,
      error,
    },
  ]
}
