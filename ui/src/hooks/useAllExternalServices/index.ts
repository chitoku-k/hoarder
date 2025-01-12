import { skipToken, useSuspenseQuery } from '@apollo/client'

import type { AllExternalServicesQuery } from '@/graphql/AllExternalServices'
import { AllExternalServicesDocument } from '@/graphql/AllExternalServices'

type AllExternalServices = AllExternalServicesQuery['allExternalServices']

export function useAllExternalServices(): AllExternalServices {
  const { data } = useSuspenseQuery(AllExternalServicesDocument)
  return data.allExternalServices
}

export function useAllExternalServicesSkip(): AllExternalServices {
  useSuspenseQuery(AllExternalServicesDocument, skipToken)
  return []
}
