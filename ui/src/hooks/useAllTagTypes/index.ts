import type { SkipToken } from '@apollo/client/react'
import { useSuspenseQuery } from '@apollo/client/react'

import type { AllTagTypesQuery } from '@/graphql/AllTagTypes'
import { AllTagTypesDocument } from '@/graphql/AllTagTypes'

type AllTagTypes = AllTagTypesQuery['allTagTypes']

export function useAllTagTypes(variables?: null | SkipToken): AllTagTypes | null {
  const { data } = useSuspenseQuery(AllTagTypesDocument, variables ?? {})
  return data?.allTagTypes ?? null
}
