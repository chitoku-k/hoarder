import { useCallback } from 'react'
import type { ApolloError } from '@apollo/client'
import { useMutation } from '@apollo/client'

import type { AttachTagMutation, AttachTagMutationVariables } from '@/graphql/AttachTag'
import { AttachTagDocument } from '@/graphql/AttachTag'
import { TagDocument } from '@/graphql/Tag'
import { AllTagsDocument, TagsDocument } from '@/graphql/Tags'

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
