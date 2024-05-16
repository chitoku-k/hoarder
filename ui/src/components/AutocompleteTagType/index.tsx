'use client'

import type { FunctionComponent } from 'react'
import { ErrorBoundary } from 'react-error-boundary'

import type { AutocompleteTagTypeBodyProps } from '@/components/AutocompleteTagTypeBody'
import AutocompleteTagTypeBody from '@/components/AutocompleteTagTypeBody'
import AutocompleteTagTypeError from '@/components/AutocompleteTagTypeError'

const AutocompleteTagType: FunctionComponent<AutocompleteTagTypeProps> = ({
  ...props
}) => (
  <ErrorBoundary fallback={<AutocompleteTagTypeError />}>
    <AutocompleteTagTypeBody {...props} />
  </ErrorBoundary>
)

export type AutocompleteTagTypeProps = AutocompleteTagTypeBodyProps

export default AutocompleteTagType
