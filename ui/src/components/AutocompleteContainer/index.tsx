import type { FunctionComponent } from 'react'
import { ErrorBoundary } from 'react-error-boundary'

import type { AutocompleteContainerBodyProps } from '@/components/AutocompleteContainerBody'
import AutocompleteContainerBody from '@/components/AutocompleteContainerBody'
import AutocompleteContainerError from '@/components/AutocompleteContainerError'

const AutocompleteContainer: FunctionComponent<AutocompleteContainerProps> = ({
  ...props
}) => (
  <ErrorBoundary fallback={<AutocompleteContainerError />}>
    <AutocompleteContainerBody {...props} />
  </ErrorBoundary>
)

export type AutocompleteContainerProps = AutocompleteContainerBodyProps

export default AutocompleteContainer
