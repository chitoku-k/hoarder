import { useCallback } from 'react'
import type { ApolloError } from '@apollo/client'
import { useMutation } from '@apollo/client'

import { MediumDocument } from '@/hooks'

import type { UpdateMediumMutation, UpdateMediumMutationVariables } from './documents.generated'
import { UpdateMediumDocument } from './documents.generated'
export { UpdateMediumDocument } from './documents.generated'

type UpdateMedium = UpdateMediumMutation['updateMedium']

export function useUpdateMedium(): [
  (variables: UpdateMediumMutationVariables) => Promise<UpdateMedium>,
  { data?: UpdateMedium, loading: boolean, error?: ApolloError },
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
      const { data } = await updateMedium({
        variables,
      })
      return data?.updateMedium!
    }, [ updateMedium ]),
    {
      data: data?.updateMedium,
      loading,
      error,
    },
  ]
}
