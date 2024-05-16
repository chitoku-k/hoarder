import { skipToken, useSuspenseQuery } from '@apollo/client'

import type { SourceQuery, SourceQueryVariables } from './documents.generated'
import { SourceDocument } from './documents.generated'
export { SourceDocument } from './documents.generated'

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
