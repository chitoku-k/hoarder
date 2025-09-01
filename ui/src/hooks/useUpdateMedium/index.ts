import { useCallback } from 'react'
import type { ErrorLike } from '@apollo/client'
import { useMutation } from '@apollo/client/react'

import { MediumDocument } from '@/graphql/Medium'
import type { UpdateMediumMutation, UpdateMediumMutationVariables } from '@/graphql/UpdateMedium'
import { UpdateMediumDocument } from '@/graphql/UpdateMedium'

type UpdateMedium = UpdateMediumMutation['updateMedium']

export function useUpdateMedium(): [
  (variables: UpdateMediumMutationVariables) => Promise<UpdateMedium>,
  { data?: UpdateMedium, loading: boolean, error?: ErrorLike },
] {
  const [ updateMedium, { data, loading, error } ] = useMutation(UpdateMediumDocument, {
    update(cache, { data }) {
      if (!data?.updateMedium) {
        return
      }

      cache.writeQuery({
        query: MediumDocument,
        data: {
          media: [
            data.updateMedium
          ]
        },
        variables: {
          id: data.updateMedium.id,
        },
      })
    },
  })

  return [
    useCallback(async (variables: UpdateMediumMutationVariables) => {
      const { data, error } = await updateMedium({
        variables,
      })
      if (!data) {
        throw new Error('invalid data', { cause: error })
      }
      return data.updateMedium
    }, [ updateMedium ]),
    {
      data: data?.updateMedium,
      loading,
      error,
    },
  ]
}
