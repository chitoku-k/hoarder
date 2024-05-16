import { useCallback } from 'react'
import type { ApolloError } from '@apollo/client'
import { useMutation } from '@apollo/client'

import { AllExternalServicesDocument } from '@/hooks'

import type { UpdateExternalServiceMutation, UpdateExternalServiceMutationVariables } from './documents.generated'
import { UpdateExternalServiceDocument } from './documents.generated'
export { UpdateExternalServiceDocument } from './documents.generated'

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
