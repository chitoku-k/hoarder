import type { FunctionComponent } from 'react'
import { Suspense } from 'react'
import { ErrorBoundary } from 'react-error-boundary'

import type { MediumListViewBodyProps } from '@/components/MediumListViewBody'
import MediumListViewBody from '@/components/MediumListViewBody'
import MediumListViewError from '@/components/MediumListViewError'
import MediumListViewLoading from '@/components/MediumListViewLoading'

const MediumListView: FunctionComponent<MediumListViewProps> = ({
  ...props
}) => (
  <ErrorBoundary fallback={<MediumListViewError />}>
    <Suspense fallback={<MediumListViewLoading />}>
      <MediumListViewBody {...props} />
    </Suspense>
  </ErrorBoundary>
)

export type MediumListViewProps = MediumListViewBodyProps

export default MediumListView
