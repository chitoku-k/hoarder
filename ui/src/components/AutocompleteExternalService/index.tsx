import type { FunctionComponent } from 'react'
import { ErrorBoundary } from 'react-error-boundary'

import type { AutocompleteExternalServiceBodyProps } from '@/components/AutocompleteExternalServiceBody'
import AutocompleteExternalServiceBody from '@/components/AutocompleteExternalServiceBody'
import AutocompleteExternalServiceError from '@/components/AutocompleteExternalServiceError'

const AutocompleteExternalService: FunctionComponent<AutocompleteExternalServiceProps> = ({
  ...props
}) => (
  <ErrorBoundary fallback={<AutocompleteExternalServiceError />}>
    <AutocompleteExternalServiceBody {...props} />
  </ErrorBoundary>
)

export type AutocompleteExternalServiceProps = AutocompleteExternalServiceBodyProps

export default AutocompleteExternalService
