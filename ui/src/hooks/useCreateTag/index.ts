import { useCallback } from 'react'
import type { ErrorLike } from '@apollo/client'
import { useMutation } from '@apollo/client/react'

import type { CreateTagMutation, CreateTagMutationVariables } from '@/graphql/CreateTag'
import { CreateTagDocument } from '@/graphql/CreateTag'
import { AllTagsDocument, TagsDocument } from '@/graphql/Tags'

type CreateTag = CreateTagMutation['createTag']

export function useCreateTag(): [
  (variables: CreateTagMutationVariables) => Promise<CreateTag>,
  { data?: CreateTag, loading: boolean, error?: ErrorLike },
] {
  const [ createTag, { data, loading, error } ] = useMutation(CreateTagDocument)
  return [
    useCallback(async (variables: CreateTagMutationVariables) => {
      const { data, error } = await createTag({
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
      return data.createTag
    }, [ createTag ]),
    {
      data: data?.createTag,
      loading,
      error,
    },
  ]
}
