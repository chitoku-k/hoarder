'use client'

import type { FunctionComponent } from 'react'
import { Suspense } from 'react'
import { ErrorBoundary } from 'react-error-boundary'
import Stack from '@mui/material/Stack'

import type { MediumListViewBodyProps } from '@/components/MediumListViewBody'
import MediumListViewBody from '@/components/MediumListViewBody'
import MediumListViewError from '@/components/MediumListViewError'
import MediumListViewLoading from '@/components/MediumListViewLoading'

import styles from './styles.module.scss'

const MediumListView: FunctionComponent<MediumListViewProps> = ({
  ...props
}) => (
  <Stack className={styles.container}>
    <ErrorBoundary fallback={<MediumListViewError />}>
      <Suspense fallback={<MediumListViewLoading />}>
        <MediumListViewBody {...props} />
      </Suspense>
    </ErrorBoundary>
  </Stack>
)

export type MediumListViewProps = MediumListViewBodyProps

export default MediumListView
