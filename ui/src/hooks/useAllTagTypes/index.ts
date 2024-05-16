import { skipToken, useSuspenseQuery } from '@apollo/client'

import type { AllTagTypesQuery } from './documents.generated'
import { AllTagTypesDocument } from './documents.generated'
export { AllTagTypesDocument } from './documents.generated'

type AllTagTypes = AllTagTypesQuery['allTagTypes']

export function useAllTagTypes(): AllTagTypes {
  const { data } = useSuspenseQuery(AllTagTypesDocument)
  return data.allTagTypes
}

export function useAllTagTypesSkip(): AllTagTypes {
  useSuspenseQuery(AllTagTypesDocument, skipToken)
  return []
}
