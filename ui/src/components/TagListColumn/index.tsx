'use client'

import type { FunctionComponent, ReactNode } from 'react'
import { Suspense, useCallback } from 'react'
import { ErrorBoundary } from 'react-error-boundary'
import type { GridDefaultBreakpoints } from '@mui/system/Unstable_Grid'
import Grid from '@mui/material/Unstable_Grid2'

import TagListColumnError from '@/components/TagListColumnError'
import TagListColumnLoading from '@/components/TagListColumnLoading'
export type { TagColumn, TagColumnSelectable } from '@/components/TagListColumnBodyList'

const TagListColumn: FunctionComponent<TagListColumnProps> = ({
  focus,
  children,
  ...props
}) => {
  const ref = useCallback((node: HTMLElement | null) => {
    if (!focus || !node?.parentElement) {
      return
    }

    const parent = node.parentElement
    const self = node

    const observer = new ResizeObserver(() => {
      const parentRects = parent.getBoundingClientRect()
      const selfRects = self.getBoundingClientRect()

      const scrollLeft = parent.scrollLeft + selfRects.left - parentRects.left
      const scrollOffset = parentRects.width - selfRects.width

      parent.scrollTo({
        left: Math.max(0, scrollLeft - scrollOffset),
        behavior: 'smooth',
      })
    })

    observer.observe(parent)
    return () => {
      observer.disconnect()
    }
  }, [ focus ])

  return (
    <Grid ref={ref} {...props}>
      <ErrorBoundary fallback={<TagListColumnError />}>
        <Suspense fallback={<TagListColumnLoading />}>
          {children}
        </Suspense>
      </ErrorBoundary>
    </Grid>
  )
}

export interface TagListColumnProps extends GridDefaultBreakpoints {
  className?: string
  focus?: boolean
  children?: ReactNode
}

export default TagListColumn
