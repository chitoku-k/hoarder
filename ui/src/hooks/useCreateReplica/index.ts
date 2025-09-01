import { useCallback } from 'react'
import type { ErrorLike } from '@apollo/client'
import { useMutation } from '@apollo/client/react'

import type { CreateReplicaMutation, CreateReplicaMutationVariables } from '@/graphql/CreateReplica'
import { CreateReplicaDocument } from '@/graphql/CreateReplica'

type CreateReplica = CreateReplicaMutation['createReplica']

export function useCreateReplica(): [
  (variables: CreateReplicaMutationVariables, fetchOptions?: CreateReplicaOptions) => Promise<CreateReplica>,
  { data?: CreateReplica, loading: boolean, error?: ErrorLike },
] {
  const [ createReplica, { data, loading, error } ] = useMutation(CreateReplicaDocument)
  return [
    useCallback(async (variables: CreateReplicaMutationVariables, fetchOptions: CreateReplicaOptions = {}) => {
      const { data, error } = await createReplica({
        variables,
        context: {
          fetchOptions,
        },
      })
      if (!data) {
        throw new Error('invalid data', { cause: error })
      }
      return data.createReplica
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
