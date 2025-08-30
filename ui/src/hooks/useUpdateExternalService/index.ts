import { useCallback } from 'react'
import type { ErrorLike } from '@apollo/client'
import { useMutation } from '@apollo/client/react'

import { AllExternalServicesDocument } from '@/graphql/AllExternalServices'
import type { UpdateExternalServiceMutation, UpdateExternalServiceMutationVariables } from '@/graphql/UpdateExternalService'
import { UpdateExternalServiceDocument } from '@/graphql/UpdateExternalService'

type UpdateExternalService = UpdateExternalServiceMutation['updateExternalService']

export function useUpdateExternalService(): [
  (variables: UpdateExternalServiceMutationVariables) => Promise<UpdateExternalService>,
  { data?: UpdateExternalService, loading: boolean, error?: ErrorLike },
] {
  const [ updateExternalService, { data, loading, error } ] = useMutation(UpdateExternalServiceDocument)
  return [
    useCallback(async (variables: UpdateExternalServiceMutationVariables) => {
      const { data, error } = await updateExternalService({
        variables,
        awaitRefetchQueries: true,
        refetchQueries: [
          AllExternalServicesDocument,
        ],
      })
      if (!data) {
        throw error
      }
      return data.updateExternalService
    }, [ updateExternalService ]),
    {
      data: data?.updateExternalService,
      loading,
      error,
    },
  ]
}
