'use client'

import type { FunctionComponent } from 'react'
import { ErrorBoundary } from 'react-error-boundary'

import type { AutocompleteSourceBodyProps } from '@/components/AutocompleteSourceBody'
import AutocompleteSourceBody from '@/components/AutocompleteSourceBody'
import AutocompleteSourceError from '@/components/AutocompleteSourceError'

const AutocompleteSource: FunctionComponent<AutocompleteSourceProps> = ({
  ...props
}) => (
  <ErrorBoundary fallback={<AutocompleteSourceError />}>
    <AutocompleteSourceBody {...props} />
  </ErrorBoundary>
)

export type AutocompleteSourceProps = AutocompleteSourceBodyProps

export default AutocompleteSource
