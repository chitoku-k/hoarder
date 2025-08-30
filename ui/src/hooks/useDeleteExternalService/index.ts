import { useCallback } from 'react'
import type { ErrorLike } from '@apollo/client'
import { useMutation } from '@apollo/client/react'

import { AllExternalServicesDocument } from '@/graphql/AllExternalServices'
import type { DeleteExternalServiceMutation, DeleteExternalServiceMutationVariables } from '@/graphql/DeleteExternalService'
import { DeleteExternalServiceDocument } from '@/graphql/DeleteExternalService'

type DeleteExternalService = DeleteExternalServiceMutation['deleteExternalService']

export function useDeleteExternalService(): [
  (variables: DeleteExternalServiceMutationVariables) => Promise<DeleteExternalService>,
  { data?: DeleteExternalService, loading: boolean, error?: ErrorLike },
] {
  const [ deleteExternalService, { data, loading, error } ] = useMutation(DeleteExternalServiceDocument)
  return [
    useCallback(async (variables: DeleteExternalServiceMutationVariables) => {
      const { data, error } = await deleteExternalService({
        variables,
        awaitRefetchQueries: true,
        refetchQueries: [
          AllExternalServicesDocument,
        ],
      })
      if (!data) {
        throw error
      }
      return data.deleteExternalService
    }, [ deleteExternalService ]),
    {
      data: data?.deleteExternalService,
      loading,
      error,
    },
  ]
}
