import { useCallback } from 'react'
import type { ErrorLike } from '@apollo/client'
import { useMutation } from '@apollo/client/react'

import { AllTagTypesDocument } from '@/graphql/AllTagTypes'
import type { CreateTagTypeMutation, CreateTagTypeMutationVariables } from '@/graphql/CreateTagType'
import { CreateTagTypeDocument } from '@/graphql/CreateTagType'

type CreateTagType = CreateTagTypeMutation['createTagType']

export function useCreateTagType(): [
  (variables: CreateTagTypeMutationVariables) => Promise<CreateTagType>,
  { data?: CreateTagType, loading: boolean, error?: ErrorLike },
] {
  const [ createTagType, { data, loading, error } ] = useMutation(CreateTagTypeDocument)
  return [
    useCallback(async (variables: CreateTagTypeMutationVariables) => {
      const { data, error } = await createTagType({
        variables,
        awaitRefetchQueries: true,
        refetchQueries: [
          AllTagTypesDocument,
        ],
      })
      if (!data) {
        throw error
      }
      return data.createTagType
    }, [ createTagType ]),
    {
      data: data?.createTagType,
      loading,
      error,
    },
  ]
}
