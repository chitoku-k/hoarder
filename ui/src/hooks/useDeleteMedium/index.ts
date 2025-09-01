import { useCallback } from 'react'
import type { ErrorLike } from '@apollo/client'
import { useMutation } from '@apollo/client/react'

import type { DeleteMediumMutation, DeleteMediumMutationVariables } from '@/graphql/DeleteMedium'
import { DeleteMediumDocument } from '@/graphql/DeleteMedium'

type DeleteMedium = DeleteMediumMutation['deleteMedium']

export function useDeleteMedium(): [
  (variables: DeleteMediumMutationVariables) => Promise<DeleteMedium>,
  { data?: DeleteMedium, loading: boolean, error?: ErrorLike },
] {
  const [ deleteMedium, { data, loading, error } ] = useMutation(DeleteMediumDocument, {
    update(cache, { data }) {
      if (!data?.deleteMedium.deleted) {
        return
      }

      cache.modify({
        fields: {
          allMedia(_allMedia, { DELETE }) {
            return DELETE
          },
        },
      })
    },
  })

  return [
    useCallback(async (variables: DeleteMediumMutationVariables) => {
      const { data, error } = await deleteMedium({
        variables,
      })
      if (!data) {
        throw new Error('invalid data', { cause: error })
      }
      return data.deleteMedium
    }, [ deleteMedium ]),
    {
      data: data?.deleteMedium,
      loading,
      error,
    },
  ]
}
