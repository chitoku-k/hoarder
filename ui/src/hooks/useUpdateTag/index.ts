import { useCallback } from 'react'
import type { ApolloError } from '@apollo/client'
import { useMutation } from '@apollo/client'

import { AllTagsDocument, TagDocument, TagsDocument } from '@/hooks'

import type { UpdateTagMutation, UpdateTagMutationVariables } from './documents.generated'
import { UpdateTagDocument } from './documents.generated'
export { UpdateTagDocument } from './documents.generated'

type UpdateTag = UpdateTagMutation['updateTag']

export function useUpdateTag(): [
  (variables: UpdateTagMutationVariables) => Promise<UpdateTag>,
  { data?: UpdateTag, loading: boolean, error?: ApolloError },
] {
  const [ updateTag, { data, loading, error } ] = useMutation(UpdateTagDocument)
  return [
    useCallback(async (variables: UpdateTagMutationVariables) => {
      const { data } = await updateTag({
        variables,
        awaitRefetchQueries: true,
        refetchQueries: [
          AllTagsDocument,
          TagDocument,
          TagsDocument,
        ],
      })
      return data?.updateTag!
    }, [ updateTag ]),
    {
      data: data?.updateTag,
      loading,
      error,
    },
  ]
}
