'use client'

import type { FunctionComponent } from 'react'
import { Suspense } from 'react'
import { ErrorBoundary } from 'react-error-boundary'

import type { MediumItemViewBodyProps } from '@/components/MediumItemViewBody'
import MediumItemViewBody from '@/components/MediumItemViewBody'
import MediumItemViewError from '@/components/MediumItemViewError'
import MediumItemViewLoading from '@/components/MediumItemViewLoading'

const MediumItemView: FunctionComponent<MediumItemViewProps> = ({
  ...props
}) => (
  <ErrorBoundary fallback={<MediumItemViewError />}>
    <Suspense fallback={<MediumItemViewLoading />}>
      <MediumItemViewBody {...props} />
    </Suspense>
  </ErrorBoundary>
)

export type MediumItemViewProps = MediumItemViewBodyProps

export default MediumItemView
