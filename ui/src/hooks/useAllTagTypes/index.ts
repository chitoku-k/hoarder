import { skipToken, useSuspenseQuery } from '@apollo/client/react'

import type { AllTagTypesQuery } from '@/graphql/AllTagTypes'
import { AllTagTypesDocument } from '@/graphql/AllTagTypes'

type AllTagTypes = AllTagTypesQuery['allTagTypes']

export function useAllTagTypes(): AllTagTypes {
  const { data } = useSuspenseQuery(AllTagTypesDocument)
  return data.allTagTypes
}

export function useAllTagTypesSkip(): AllTagTypes {
  useSuspenseQuery(AllTagTypesDocument, skipToken)
  return []
}
