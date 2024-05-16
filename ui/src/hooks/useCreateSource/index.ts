import { useCallback } from 'react'
import type { ApolloError } from '@apollo/client'
import { useMutation } from '@apollo/client'

import { SourceDocument } from '@/hooks'

import type { CreateSourceMutation, CreateSourceMutationVariables } from './documents.generated'
import { CreateSourceDocument } from './documents.generated'
export { CreateSourceDocument } from './documents.generated'

type CreateSource = CreateSourceMutation['createSource']

export function useCreateSource(): [
  (variables: CreateSourceMutationVariables) => Promise<CreateSource>,
  { data?: CreateSource, loading: boolean, error?: ApolloError },
] {
  const [ createSource, { data, loading, error } ] = useMutation(CreateSourceDocument)
  return [
    useCallback(async (variables: CreateSourceMutationVariables) => {
      const { data } = await createSource({
        variables,
        awaitRefetchQueries: true,
        refetchQueries: [
          SourceDocument,
        ],
      })
      return data?.createSource!
    }, [ createSource ]),
    {
      data: data?.createSource,
      loading,
      error,
    },
  ]
}
