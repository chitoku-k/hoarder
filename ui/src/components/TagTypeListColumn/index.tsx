'use client'

import type { FunctionComponent, ReactNode } from 'react'
import { Suspense } from 'react'
import { ErrorBoundary } from 'react-error-boundary'
import type { GridProps } from '@mui/material/Grid'
import Grid from '@mui/material/Grid'

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

export interface TagTypeListColumnProps extends GridProps {
  className?: string
  children?: ReactNode
}

export default TagTypeListColumn
