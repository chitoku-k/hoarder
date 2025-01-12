import { skipToken, useSuspenseQuery } from '@apollo/client'

import type { ObjectsQuery, ObjectsQueryVariables } from '@/graphql/Objects'
import { ObjectsDocument } from '@/graphql/Objects'

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
