import { useCallback } from 'react'
import type { ApolloError } from '@apollo/client'
import { useMutation } from '@apollo/client'

import { AllExternalServicesDocument } from '@/graphql/AllExternalServices'
import type { UpdateExternalServiceMutation, UpdateExternalServiceMutationVariables } from '@/graphql/UpdateExternalService'
import { UpdateExternalServiceDocument } from '@/graphql/UpdateExternalService'

type UpdateExternalService = UpdateExternalServiceMutation['updateExternalService']

export function useUpdateExternalService(): [
  (variables: UpdateExternalServiceMutationVariables) => Promise<UpdateExternalService>,
  { data?: UpdateExternalService, loading: boolean, error?: ApolloError },
] {
  const [ updateExternalService, { data, loading, error } ] = useMutation(UpdateExternalServiceDocument)
  return [
    useCallback(async (variables: UpdateExternalServiceMutationVariables) => {
      const { data } = await updateExternalService({
        variables,
        awaitRefetchQueries: true,
        refetchQueries: [
          AllExternalServicesDocument,
        ],
      })
      return data?.updateExternalService!
    }, [ updateExternalService ]),
    {
      data: data?.updateExternalService,
      loading,
      error,
    },
  ]
}
