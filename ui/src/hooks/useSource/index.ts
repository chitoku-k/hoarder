import { skipToken, useSuspenseQuery } from '@apollo/client'

import type { SourceQuery, SourceQueryVariables } from '@/graphql/Source'
import { SourceDocument } from '@/graphql/Source'

type Source = SourceQuery['source']

export function useSource(variables: SourceQueryVariables): Source {
  const { data } = useSuspenseQuery(SourceDocument, {
    variables,
    fetchPolicy: 'no-cache',
  })
  return data.source
}

export function useSourceSkip(): null {
  useSuspenseQuery(SourceDocument, skipToken)
  return null
}
