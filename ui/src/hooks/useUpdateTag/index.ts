import { useCallback } from 'react'
import type { ErrorLike } from '@apollo/client'
import { useMutation } from '@apollo/client/react'

import type { UpdateTagMutation, UpdateTagMutationVariables } from '@/graphql/UpdateTag'
import { UpdateTagDocument } from '@/graphql/UpdateTag'
import { AllTagsDocument, TagsDocument } from '@/graphql/Tags'

type UpdateTag = UpdateTagMutation['updateTag']

export function useUpdateTag(): [
  (variables: UpdateTagMutationVariables) => Promise<UpdateTag>,
  { data?: UpdateTag, loading: boolean, error?: ErrorLike },
] {
  const [ updateTag, { data, loading, error } ] = useMutation(UpdateTagDocument)
  return [
    useCallback(async (variables: UpdateTagMutationVariables) => {
      const { data, error } = await updateTag({
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
      return data.updateTag
    }, [ updateTag ]),
    {
      data: data?.updateTag,
      loading,
      error,
    },
  ]
}
