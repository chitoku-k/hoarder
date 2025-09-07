import { useCallback } from 'react'
import type { ErrorLike } from '@apollo/client'
import { useMutation } from '@apollo/client/react'

import type { CreateSourceMutation, CreateSourceMutationVariables } from '@/graphql/CreateSource'
import { CreateSourceDocument } from '@/graphql/CreateSource'

type CreateSource = CreateSourceMutation['createSource']

export function useCreateSource(): [
  (variables: CreateSourceMutationVariables) => Promise<CreateSource>,
  { data?: CreateSource, loading: boolean, error?: ErrorLike },
] {
  const [ createSource, { data, loading, error } ] = useMutation(CreateSourceDocument)
  return [
    useCallback(async (variables: CreateSourceMutationVariables) => {
      const { data, error } = await createSource({
        variables,
        awaitRefetchQueries: true,
      })
      if (!data) {
        throw new Error('invalid data', { cause: error })
      }
      return data.createSource
    }, [ createSource ]),
    {
      data: data?.createSource,
      loading,
      error,
    },
  ]
}
