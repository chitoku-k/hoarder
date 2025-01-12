import { useCallback } from 'react'
import type { ApolloError } from '@apollo/client'
import { useMutation } from '@apollo/client'

import type { DeleteReplicaMutation, DeleteReplicaMutationVariables } from '@/graphql/DeleteReplica'
import { DeleteReplicaDocument } from '@/graphql/DeleteReplica'

type DeleteReplica = DeleteReplicaMutation['deleteReplica']

export function useDeleteReplica(): [
  (variables: DeleteReplicaMutationVariables) => Promise<DeleteReplica>,
  { data?: DeleteReplica, loading: boolean, error?: ApolloError },
] {
  const [ deleteReplica, { data, loading, error } ] = useMutation(DeleteReplicaDocument)
  return [
    useCallback(async (variables: DeleteReplicaMutationVariables) => {
      const { data } = await deleteReplica({
        variables,
      })
      return data?.deleteReplica!
    }, [ deleteReplica ]),
    {
      data: data?.deleteReplica,
      loading,
      error,
    },
  ]
}
