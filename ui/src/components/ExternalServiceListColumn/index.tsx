'use client'

import type { FunctionComponent, ReactNode } from 'react'
import { Suspense } from 'react'
import { ErrorBoundary } from 'react-error-boundary'
import type { GridDefaultBreakpoints } from '@mui/system/Unstable_Grid'
import Grid from '@mui/material/Unstable_Grid2'

import ExternalServiceListColumnError from '@/components/ExternalServiceListColumnError'
import ExternalServiceListColumnLoading from '@/components/ExternalServiceListColumnLoading'
export type { ExternalServiceColumn } from '@/components/ExternalServiceListColumnBodyList'

const ExternalServiceListColumn: FunctionComponent<ExternalServiceListColumnProps> = ({
  children,
  ...props
}) => (
  <Grid {...props}>
    <ErrorBoundary fallback={<ExternalServiceListColumnError />}>
      <Suspense fallback={<ExternalServiceListColumnLoading />}>
        {children}
      </Suspense>
    </ErrorBoundary>
  </Grid>
)

export interface ExternalServiceListColumnProps extends GridDefaultBreakpoints {
  className?: string
  children?: ReactNode
}

export default ExternalServiceListColumn
