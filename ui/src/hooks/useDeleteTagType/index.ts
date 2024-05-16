import { useCallback } from 'react'
import type { ApolloError } from '@apollo/client'
import { useMutation } from '@apollo/client'

import { AllTagTypesDocument } from '@/hooks'

import type { DeleteTagTypeMutation, DeleteTagTypeMutationVariables } from './documents.generated'
import { DeleteTagTypeDocument } from './documents.generated'
export { DeleteTagTypeDocument } from './documents.generated'

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
