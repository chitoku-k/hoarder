import type { SkipToken } from '@apollo/client/react'
import { useSuspenseQuery } from '@apollo/client/react'

import type { SourceQuery, SourceQueryVariables } from '@/graphql/Source'
import { SourceDocument } from '@/graphql/Source'

type Source = SourceQuery['source']

export function useSource(variables: SourceQueryVariables | SkipToken): Source | null {
  const options = typeof variables === 'symbol'
    ? variables
    : { variables, fetchPolicy: 'no-cache' } satisfies useSuspenseQuery.Options<SourceQueryVariables>

  const { data } = useSuspenseQuery(SourceDocument, options)
  return data?.source ?? null
}
