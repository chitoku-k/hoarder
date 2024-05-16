import { useCallback } from 'react'
import type { ApolloError } from '@apollo/client'
import { useMutation } from '@apollo/client'

import { AllExternalServicesDocument } from '@/hooks'

import type { DeleteExternalServiceMutation, DeleteExternalServiceMutationVariables } from './documents.generated'
import { DeleteExternalServiceDocument } from './documents.generated'
export { DeleteExternalServiceDocument } from './documents.generated'

type DeleteExternalService = DeleteExternalServiceMutation['deleteExternalService']

export function useDeleteExternalService(): [
  (variables: DeleteExternalServiceMutationVariables) => Promise<DeleteExternalService>,
  { data?: DeleteExternalService, loading: boolean, error?: ApolloError },
] {
  const [ deleteExternalService, { data, loading, error } ] = useMutation(DeleteExternalServiceDocument)
  return [
    useCallback(async (variables: DeleteExternalServiceMutationVariables) => {
      const { data } = await deleteExternalService({
        variables,
        awaitRefetchQueries: true,
        refetchQueries: [
          AllExternalServicesDocument,
        ],
      })
      return data?.deleteExternalService!
    }, [ deleteExternalService ]),
    {
      data: data?.deleteExternalService,
      loading,
      error,
    },
  ]
}
