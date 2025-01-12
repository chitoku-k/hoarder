import { useCallback } from 'react'
import type { ApolloError } from '@apollo/client'
import { useMutation } from '@apollo/client'

import { AllTagTypesDocument } from '@/graphql/AllTagTypes'
import type { DeleteTagTypeMutation, DeleteTagTypeMutationVariables } from '@/graphql/DeleteTagType'
import { DeleteTagTypeDocument } from '@/graphql/DeleteTagType'

type DeleteTagType = DeleteTagTypeMutation['deleteTagType']

export function useDeleteTagType(): [
  (variables: DeleteTagTypeMutationVariables) => Promise<DeleteTagType>,
  { data?: DeleteTagType, loading: boolean, error?: ApolloError },
] {
  const [ deleteTagType, { data, loading, error } ] = useMutation(DeleteTagTypeDocument)
  return [
    useCallback(async (variables: DeleteTagTypeMutationVariables) => {
      const { data } = await deleteTagType({
        variables,
        awaitRefetchQueries: true,
        refetchQueries: [
          AllTagTypesDocument,
        ],
      })
      return data?.deleteTagType!
    }, [ deleteTagType ]),
    {
      data: data?.deleteTagType,
      loading,
      error,
    },
  ]
}
