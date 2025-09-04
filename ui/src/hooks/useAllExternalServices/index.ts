import type { SkipToken } from '@apollo/client/react'
import { useSuspenseQuery } from '@apollo/client/react'

import type { AllExternalServicesQuery } from '@/graphql/AllExternalServices'
import { AllExternalServicesDocument } from '@/graphql/AllExternalServices'

type AllExternalServices = AllExternalServicesQuery['allExternalServices']

export function useAllExternalServices(variables?: null | SkipToken): AllExternalServices | null {
  const { data } = useSuspenseQuery(AllExternalServicesDocument, variables ?? {})
  return data?.allExternalServices ?? null
}
