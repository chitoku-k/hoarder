import { useCallback } from 'react'
import type { ApolloError } from '@apollo/client'
import { useMutation } from '@apollo/client'

import type { DeleteMediumMutation, DeleteMediumMutationVariables } from './documents.generated'
import { DeleteMediumDocument } from './documents.generated'
export { DeleteMediumDocument } from './documents.generated'

type DeleteMedium = DeleteMediumMutation['deleteMedium']

export function useDeleteMedium(): [
  (variables: DeleteMediumMutationVariables) => Promise<DeleteMedium>,
  { data?: DeleteMedium, loading: boolean, error?: ApolloError },
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
      const { data } = await deleteMedium({
        variables,
      })
      return data?.deleteMedium!
    }, [ deleteMedium ]),
    {
      data: data?.deleteMedium,
      loading,
      error,
    },
  ]
}
