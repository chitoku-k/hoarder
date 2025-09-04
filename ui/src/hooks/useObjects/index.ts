import type { SkipToken } from '@apollo/client/react'
import { useSuspenseQuery } from '@apollo/client/react'

import type { ObjectsQuery, ObjectsQueryVariables } from '@/graphql/Objects'
import { ObjectsDocument } from '@/graphql/Objects'

type Objects = ObjectsQuery['objects']

export function useObjects(variables: ObjectsQueryVariables | SkipToken): Objects {
  const options = typeof variables === 'symbol'
    ? variables
    : { variables, errorPolicy: 'ignore' } satisfies useSuspenseQuery.Options<ObjectsQueryVariables>

  const { data } = useSuspenseQuery(ObjectsDocument, options)
  return data?.objects ?? []
}
