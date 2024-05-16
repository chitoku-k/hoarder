import { useCallback } from 'react'
import type { ApolloError, Reference } from '@apollo/client'
import { useMutation } from '@apollo/client'

import type { AllMediaQuery } from '@/hooks'
import { MediumDocument } from '@/hooks'

import type { CreateMediumMutation, CreateMediumMutationVariables } from './documents.generated'
import { CreateMediumDocument } from './documents.generated'
export { CreateMediumDocument } from './documents.generated'

type CreateMedium = CreateMediumMutation['createMedium']
type MediumNode = AllMediaQuery['allMedia']['edges'][number]['node']

export function useCreateMedium(): [
  (variables: CreateMediumMutationVariables) => Promise<CreateMedium>,
  { data?: CreateMedium, loading: boolean, error?: ApolloError },
] {
  const [ createMedium, { data, loading, error } ] = useMutation(CreateMediumDocument, {
    update(cache, { data }) {
      if (!data?.createMedium) {
        return
      }

      cache.writeQuery({
        query: MediumDocument,
        data: {
          media: [
            data.createMedium
          ]
        },
        variables: {
          id: data.createMedium.id,
        },
      })

      cache.modify({
        fields: {
          allMedia(allMedia: Reference | AllMediaQuery['allMedia'], { isReference, toReference }) {
            if (isReference(allMedia)) {
              return allMedia
            }
            return {
              ...allMedia,
              edges: [
                {
                  __typename: 'MediumEdge',
                  node: toReference(data.createMedium) as unknown as MediumNode,
                },
                ...allMedia.edges,
              ],
            }
          },
        },
      })
    },
  })

  return [
    useCallback(async (variables: CreateMediumMutationVariables) => {
      const { data } = await createMedium({
        variables,
      })
      return data?.createMedium!
    }, [ createMedium ]),
    {
      data: data?.createMedium,
      loading,
      error,
    },
  ]
}
