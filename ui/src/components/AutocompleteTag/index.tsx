'use client'

import type { FunctionComponent } from 'react'
import { ErrorBoundary } from 'react-error-boundary'

import type { AutocompleteTagBodyProps } from '@/components/AutocompleteTagBody'
import AutocompleteTagBody from '@/components/AutocompleteTagBody'
import AutocompleteTagError from '@/components/AutocompleteTagError'

const AutocompleteTag: FunctionComponent<AutocompleteTagProps> = ({
  ...props
}) => (
  <ErrorBoundary fallback={<AutocompleteTagError />}>
    <AutocompleteTagBody {...props} />
  </ErrorBoundary>
)

export type AutocompleteTagProps = AutocompleteTagBodyProps

export default AutocompleteTag
