import { useCallback } from 'react'
import type { ErrorLike } from '@apollo/client'
import { useMutation } from '@apollo/client/react'

import type { DeleteReplicaMutation, DeleteReplicaMutationVariables } from '@/graphql/DeleteReplica'
import { DeleteReplicaDocument } from '@/graphql/DeleteReplica'

type DeleteReplica = DeleteReplicaMutation['deleteReplica']

export function useDeleteReplica(): [
  (variables: DeleteReplicaMutationVariables) => Promise<DeleteReplica>,
  { data?: DeleteReplica, loading: boolean, error?: ErrorLike },
] {
  const [ deleteReplica, { data, loading, error } ] = useMutation(DeleteReplicaDocument)
  return [
    useCallback(async (variables: DeleteReplicaMutationVariables) => {
      const { data, error } = await deleteReplica({
        variables,
      })
      if (!data) {
        throw new Error('invalid data', { cause: error })
      }
      return data.deleteReplica
    }, [ deleteReplica ]),
    {
      data: data?.deleteReplica,
      loading,
      error,
    },
  ]
}
