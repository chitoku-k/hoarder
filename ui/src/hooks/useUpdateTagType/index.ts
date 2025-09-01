import { useCallback } from 'react'
import type { ErrorLike } from '@apollo/client'
import { useMutation } from '@apollo/client/react'

import { AllTagTypesDocument } from '@/graphql/AllTagTypes'
import type { UpdateTagTypeMutation, UpdateTagTypeMutationVariables } from '@/graphql/UpdateTagType'
import { UpdateTagTypeDocument } from '@/graphql/UpdateTagType'

type UpdateTagType = UpdateTagTypeMutation['updateTagType']

export function useUpdateTagType(): [
  (variables: UpdateTagTypeMutationVariables) => Promise<UpdateTagType>,
  { data?: UpdateTagType, loading: boolean, error?: ErrorLike },
] {
  const [ updateTagType, { data, loading, error } ] = useMutation(UpdateTagTypeDocument)
  return [
    useCallback(async (variables: UpdateTagTypeMutationVariables) => {
      const { data, error } = await updateTagType({
        variables,
        awaitRefetchQueries: true,
        refetchQueries: [
          AllTagTypesDocument,
        ],
      })
      if (!data) {
        throw new Error('invalid data', { cause: error })
      }
      return data.updateTagType
    }, [ updateTagType ]),
    {
      data: data?.updateTagType,
      loading,
      error,
    },
  ]
}
