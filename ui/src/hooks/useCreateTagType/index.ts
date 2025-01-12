import { useCallback } from 'react'
import type { ApolloError } from '@apollo/client'
import { useMutation } from '@apollo/client'

import { AllTagTypesDocument } from '@/graphql/AllTagTypes'
import type { CreateTagTypeMutation, CreateTagTypeMutationVariables } from '@/graphql/CreateTagType'
import { CreateTagTypeDocument } from '@/graphql/CreateTagType'

type CreateTagType = CreateTagTypeMutation['createTagType']

export function useCreateTagType(): [
  (variables: CreateTagTypeMutationVariables) => Promise<CreateTagType>,
  { data?: CreateTagType, loading: boolean, error?: ApolloError },
] {
  const [ createTagType, { data, loading, error } ] = useMutation(CreateTagTypeDocument)
  return [
    useCallback(async (variables: CreateTagTypeMutationVariables) => {
      const { data } = await createTagType({
        variables,
        awaitRefetchQueries: true,
        refetchQueries: [
          AllTagTypesDocument,
        ],
      })
      return data?.createTagType!
    }, [ createTagType ]),
    {
      data: data?.createTagType,
      loading,
      error,
    },
  ]
}
