import { useCallback } from 'react'
import type { ApolloError } from '@apollo/client'
import { useMutation } from '@apollo/client'

import type { DeleteTagMutation, DeleteTagMutationVariables } from '@/graphql/DeleteTag'
import { DeleteTagDocument } from '@/graphql/DeleteTag'
import { AllTagsDocument, TagsDocument } from '@/graphql/Tags'

type DeleteTag = DeleteTagMutation['deleteTag']

export function useDeleteTag(): [
  (variables: DeleteTagMutationVariables) => Promise<DeleteTag>,
  { data?: DeleteTag, loading: boolean, error?: ApolloError },
] {
  const [ deleteTag, { data, loading, error } ] = useMutation(DeleteTagDocument)
  return [
    useCallback(async (variables: DeleteTagMutationVariables) => {
      const { data } = await deleteTag({
        variables,
        refetchQueries: [
          AllTagsDocument,
          TagsDocument,
        ],
      })
      return data?.deleteTag!
    }, [ deleteTag ]),
    {
      data: data?.deleteTag,
      loading,
      error,
    },
  ]
}
