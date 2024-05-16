import { skipToken, useSuspenseQuery } from '@apollo/client'

import type { ObjectsQuery, ObjectsQueryVariables } from './documents.generated'
import { ObjectsDocument } from './documents.generated'
export { ObjectsDocument } from './documents.generated'
export { ObjectKind } from '@/hooks/types.generated'

type Objects = ObjectsQuery['objects']

export function useObjects(variables: ObjectsQueryVariables): Objects {
  const { data } = useSuspenseQuery(ObjectsDocument, {
    variables,
    errorPolicy: 'ignore',
  })
  return data?.objects ?? []
}

export function useObjectsSkip(): Objects {
  useSuspenseQuery(ObjectsDocument, skipToken)
  return []
}
