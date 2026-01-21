import type { FunctionComponent } from 'react'
import { ErrorBoundary } from 'react-error-boundary'

import type { AutocompleteMetadataBodyProps } from '@/components/AutocompleteMetadataBody'
import AutocompleteMetadataBody from '@/components/AutocompleteMetadataBody'
import AutocompleteMetadataError from '@/components/AutocompleteMetadataError'
export * from '@/components/AutocompleteMetadataBody'

const AutocompleteMetadata: FunctionComponent<AutocompleteMetadataProps> = ({
  ...props
}) => (
  <ErrorBoundary fallback={<AutocompleteMetadataError />}>
    <AutocompleteMetadataBody {...props} />
  </ErrorBoundary>
)

export type AutocompleteMetadataProps = AutocompleteMetadataBodyProps

export default AutocompleteMetadata
