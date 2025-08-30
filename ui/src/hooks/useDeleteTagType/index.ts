import { useCallback } from 'react'
import type { ErrorLike } from '@apollo/client'
import { useMutation } from '@apollo/client/react'

import { AllTagTypesDocument } from '@/graphql/AllTagTypes'
import type { DeleteTagTypeMutation, DeleteTagTypeMutationVariables } from '@/graphql/DeleteTagType'
import { DeleteTagTypeDocument } from '@/graphql/DeleteTagType'

type DeleteTagType = DeleteTagTypeMutation['deleteTagType']

export function useDeleteTagType(): [
  (variables: DeleteTagTypeMutationVariables) => Promise<DeleteTagType>,
  { data?: DeleteTagType, loading: boolean, error?: ErrorLike },
] {
  const [ deleteTagType, { data, loading, error } ] = useMutation(DeleteTagTypeDocument)
  return [
    useCallback(async (variables: DeleteTagTypeMutationVariables) => {
      const { data, error } = await deleteTagType({
        variables,
        awaitRefetchQueries: true,
        refetchQueries: [
          AllTagTypesDocument,
        ],
      })
      if (!data) {
        throw error
      }
      return data.deleteTagType
    }, [ deleteTagType ]),
    {
      data: data?.deleteTagType,
      loading,
      error,
    },
  ]
}
