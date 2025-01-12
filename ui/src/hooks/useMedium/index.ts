import { useSuspenseQuery } from '@apollo/client'

import type { MediumQuery, MediumQueryVariables } from '@/graphql/Medium'
import { MediumDocument } from '@/graphql/Medium'

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
