import { useCallback } from 'react'
import type { ApolloError } from '@apollo/client'
import { useMutation } from '@apollo/client'

import { AllTagTypesDocument } from '@/graphql/AllTagTypes'
import type { UpdateTagTypeMutation, UpdateTagTypeMutationVariables } from '@/graphql/UpdateTagType'
import { UpdateTagTypeDocument } from '@/graphql/UpdateTagType'

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
