'use client'

import type { FunctionComponent, ReactNode } from 'react'
import { Suspense } from 'react'
import { ErrorBoundary } from 'react-error-boundary'
import type { Grid2Props } from '@mui/material/Grid2'
import Grid from '@mui/material/Grid2'

import TagTypeListColumnError from '@/components/TagTypeListColumnError'
import TagTypeListColumnLoading from '@/components/TagTypeListColumnLoading'
export type { TagTypeColumn } from '@/components/TagTypeListColumnBodyList'

const TagTypeListColumn: FunctionComponent<TagTypeListColumnProps> = ({
  children,
  ...props
}) => (
  <Grid {...props}>
    <ErrorBoundary fallback={<TagTypeListColumnError />}>
      <Suspense fallback={<TagTypeListColumnLoading />}>
        {children}
      </Suspense>
    </ErrorBoundary>
  </Grid>
)

export interface TagTypeListColumnProps extends Grid2Props {
  className?: string
  children?: ReactNode
}

export default TagTypeListColumn
