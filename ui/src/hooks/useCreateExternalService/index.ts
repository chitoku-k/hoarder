import { useCallback } from 'react'
import type { ErrorLike } from '@apollo/client'
import { useMutation } from '@apollo/client/react'

import { AllExternalServicesDocument } from '@/graphql/AllExternalServices'
import type { CreateExternalServiceMutation, CreateExternalServiceMutationVariables } from '@/graphql/CreateExternalService'
import { CreateExternalServiceDocument } from '@/graphql/CreateExternalService'

type CreateExternalService = CreateExternalServiceMutation['createExternalService']

export function useCreateExternalService(): [
  (variables: CreateExternalServiceMutationVariables) => Promise<CreateExternalService>,
  { data?: CreateExternalService, loading: boolean, error?: ErrorLike },
] {
  const [ createExternalService, { data, loading, error } ] = useMutation(CreateExternalServiceDocument)
  return [
    useCallback(async (variables: CreateExternalServiceMutationVariables) => {
      const { data, error } = await createExternalService({
        variables,
        awaitRefetchQueries: true,
        refetchQueries: [
          AllExternalServicesDocument,
        ],
      })
      if (!data) {
        throw error
      }
      return data.createExternalService
    }, [ createExternalService ]),
    {
      data: data?.createExternalService,
      loading,
      error,
    },
  ]
}
