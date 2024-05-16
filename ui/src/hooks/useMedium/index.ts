import { useSuspenseQuery } from '@apollo/client'

import type { MediumQuery, MediumQueryVariables } from './documents.generated'
import { MediumDocument } from './documents.generated'
export { MediumDocument } from './documents.generated'

type Medium = MediumQuery['media'][number]

export function useMedium(variables: MediumQueryVariables): Medium {
  const { data } = useSuspenseQuery(MediumDocument, {
    variables,
  })
  if (!data.media[0]) {
    throw new Error('medium not found')
  }
  return data.media[0]
}
