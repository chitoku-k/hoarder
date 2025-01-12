import { useCallback } from 'react'
import type { ApolloError } from '@apollo/client'
import { useMutation } from '@apollo/client'

import type { CreateTagMutation, CreateTagMutationVariables } from '@/graphql/CreateTag'
import { CreateTagDocument } from '@/graphql/CreateTag'
import { TagDocument } from '@/graphql/Tag'
import { AllTagsDocument, TagsDocument } from '@/graphql/Tags'

type CreateTag = CreateTagMutation['createTag']

export function useCreateTag(): [
  (variables: CreateTagMutationVariables) => Promise<CreateTag>,
  { data?: CreateTag, loading: boolean, error?: ApolloError },
] {
  const [ createTag, { data, loading, error } ] = useMutation(CreateTagDocument)
  return [
    useCallback(async (variables: CreateTagMutationVariables) => {
      const { data } = await createTag({
        variables,
        awaitRefetchQueries: true,
        refetchQueries: [
          AllTagsDocument,
          TagDocument,
          TagsDocument,
        ],
      })
      return data?.createTag!
    }, [ createTag ]),
    {
      data: data?.createTag,
      loading,
      error,
    },
  ]
}
