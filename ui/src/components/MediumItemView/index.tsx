'use client'

import type { FunctionComponent } from 'react'
import { Suspense } from 'react'
import { ErrorBoundary } from 'react-error-boundary'
import Stack from '@mui/material/Stack'

import type { MediumItemViewBodyProps } from '@/components/MediumItemViewBody'
import MediumItemViewBody from '@/components/MediumItemViewBody'
import MediumItemViewError from '@/components/MediumItemViewError'
import MediumItemViewLoading from '@/components/MediumItemViewLoading'

import styles from './styles.module.scss'

const MediumItemView: FunctionComponent<MediumItemViewProps> = ({
  ...props
}) => (
  <Stack className={styles.container}>
    <ErrorBoundary fallback={<MediumItemViewError />}>
      <Suspense fallback={<MediumItemViewLoading />}>
        <MediumItemViewBody {...props} />
      </Suspense>
    </ErrorBoundary>
  </Stack>
)

export type MediumItemViewProps = MediumItemViewBodyProps

export default MediumItemView
