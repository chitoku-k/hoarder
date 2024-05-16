import { useCallback } from 'react'
import type { ApolloError } from '@apollo/client'
import { useMutation } from '@apollo/client'

import { AllExternalServicesDocument } from '@/hooks'

import type { CreateExternalServiceMutation, CreateExternalServiceMutationVariables } from './documents.generated'
import { CreateExternalServiceDocument } from './documents.generated'
export { CreateExternalServiceDocument } from './documents.generated'

type CreateExternalService = CreateExternalServiceMutation['createExternalService']

export function useCreateExternalService(): [
  (variables: CreateExternalServiceMutationVariables) => Promise<CreateExternalService>,
  { data?: CreateExternalService, loading: boolean, error?: ApolloError },
] {
  const [ createExternalService, { data, loading, error } ] = useMutation(CreateExternalServiceDocument)
  return [
    useCallback(async (variables: CreateExternalServiceMutationVariables) => {
      const { data } = await createExternalService({
        variables,
        awaitRefetchQueries: true,
        refetchQueries: [
          AllExternalServicesDocument,
        ],
      })
      return data?.createExternalService!
    }, [ createExternalService ]),
    {
      data: data?.createExternalService,
      loading,
      error,
    },
  ]
}
