import { skipToken, useSuspenseQuery } from '@apollo/client'

import type { AllExternalServicesQuery } from './documents.generated'
import { AllExternalServicesDocument } from './documents.generated'
export { AllExternalServicesDocument } from './documents.generated'

type AllExternalServices = AllExternalServicesQuery['allExternalServices']

export function useAllExternalServices(): AllExternalServices {
  const { data } = useSuspenseQuery(AllExternalServicesDocument)
  return data.allExternalServices
}

export function useAllExternalServicesSkip(): AllExternalServices {
  useSuspenseQuery(AllExternalServicesDocument, skipToken)
  return []
}
