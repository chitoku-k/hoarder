import { useCallback } from 'react'
import type { ErrorLike } from '@apollo/client'
import { useMutation } from '@apollo/client/react'

import type { AttachTagMutation, AttachTagMutationVariables } from '@/graphql/AttachTag'
import { AttachTagDocument } from '@/graphql/AttachTag'
import { AllTagsDocument, TagsDocument } from '@/graphql/Tags'

type AttachTag = AttachTagMutation['attachTag']

export function useAttachTag(): [
  (variables: AttachTagMutationVariables) => Promise<AttachTag>,
  { data?: AttachTag, loading: boolean, error?: ErrorLike },
] {
  const [ attachTag, { data, loading, error } ] = useMutation(AttachTagDocument)
  return [
    useCallback(async (variables: AttachTagMutationVariables) => {
      const { data, error } = await attachTag({
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
      return data.attachTag
    }, [ attachTag ]),
    {
      data: data?.attachTag,
      loading,
      error,
    },
  ]
}
