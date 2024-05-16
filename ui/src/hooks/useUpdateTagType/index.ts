import { useCallback } from 'react'
import type { ApolloError } from '@apollo/client'
import { useMutation } from '@apollo/client'

import { AllTagTypesDocument } from '@/hooks'

import type { UpdateTagTypeMutation, UpdateTagTypeMutationVariables } from './documents.generated'
import { UpdateTagTypeDocument } from './documents.generated'
export { UpdateTagTypeDocument } from './documents.generated'

type UpdateTagType = UpdateTagTypeMutation['updateTagType']

export function useUpdateTagType(): [
  (variables: UpdateTagTypeMutationVariables) => Promise<UpdateTagType>,
  { data?: UpdateTagType, loading: boolean, error?: ApolloError },
] {
  const [ updateTagType, { data, loading, error } ] = useMutation(UpdateTagTypeDocument)
  return [
    useCallback(async (variables: UpdateTagTypeMutationVariables) => {
      const { data } = await updateTagType({
        variables,
        awaitRefetchQueries: true,
        refetchQueries: [
          AllTagTypesDocument,
        ],
      })
      return data?.updateTagType!
    }, [ updateTagType ]),
    {
      data: data?.updateTagType,
      loading,
      error,
    },
  ]
}
