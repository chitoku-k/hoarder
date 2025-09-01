import { useCallback } from 'react'
import type { ErrorLike } from '@apollo/client'
import { useMutation } from '@apollo/client/react'

import type { DeleteTagMutation, DeleteTagMutationVariables } from '@/graphql/DeleteTag'
import { DeleteTagDocument } from '@/graphql/DeleteTag'
import { AllTagsDocument, TagsDocument } from '@/graphql/Tags'

type DeleteTag = DeleteTagMutation['deleteTag']

export function useDeleteTag(): [
  (variables: DeleteTagMutationVariables) => Promise<DeleteTag>,
  { data?: DeleteTag, loading: boolean, error?: ErrorLike },
] {
  const [ deleteTag, { data, loading, error } ] = useMutation(DeleteTagDocument)
  return [
    useCallback(async (variables: DeleteTagMutationVariables) => {
      const { data, error } = await deleteTag({
        variables,
        refetchQueries: [
          AllTagsDocument,
          TagsDocument,
        ],
      })
      if (!data) {
        throw new Error('invalid data', { cause: error })
      }
      return data.deleteTag
    }, [ deleteTag ]),
    {
      data: data?.deleteTag,
      loading,
      error,
    },
  ]
}
