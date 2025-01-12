import { useCallback } from 'react'
import type { ApolloError } from '@apollo/client'
import { useMutation } from '@apollo/client'

import type { CreateReplicaMutation, CreateReplicaMutationVariables } from '@/graphql/CreateReplica'
import { CreateReplicaDocument } from '@/graphql/CreateReplica'

type CreateReplica = CreateReplicaMutation['createReplica']

export function useCreateReplica(): [
  (variables: CreateReplicaMutationVariables, fetchOptions?: CreateReplicaOptions) => Promise<CreateReplica>,
  { data?: CreateReplica, loading: boolean, error?: ApolloError },
] {
  const [ createReplica, { data, loading, error } ] = useMutation(CreateReplicaDocument)
  return [
    useCallback(async (variables: CreateReplicaMutationVariables, fetchOptions: CreateReplicaOptions = {}) => {
      const { data } = await createReplica({
        variables,
        context: {
          fetchOptions,
        },
      })
      return data?.createReplica!
    }, [ createReplica ]),
    {
      data: data?.createReplica,
      loading,
      error,
    },
  ]
}

export interface CreateReplicaOptions {
  signal?: AbortSignal
  onUploadProgress?: (e: ProgressEvent) => void
}
