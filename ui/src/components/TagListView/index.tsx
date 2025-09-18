'use client'

import type { FunctionComponent } from 'react'
import { Suspense } from 'react'
import clsx from 'clsx'
import { ErrorBoundary } from 'react-error-boundary'
import Card from '@mui/material/Card'

import type { TagListViewBodyProps } from '@/components/TagListViewBody'
import TagListViewBody from '@/components/TagListViewBody'
import TagListViewError from '@/components/TagListViewError'
import TagListViewLoading from '@/components/TagListViewLoading'
export { ancestors } from '@/components/TagListViewBody'

import styles from './styles.module.scss'

const TagListView: FunctionComponent<TagListViewProps> = ({
  className,
  ...props
}) => (
  <Card className={clsx(styles.container, className)}>
    <ErrorBoundary fallback={<TagListViewError />}>
      <Suspense fallback={<TagListViewLoading />}>
        <TagListViewBody {...props} />
      </Suspense>
    </ErrorBoundary>
  </Card>
)

export interface TagListViewProps extends TagListViewBodyProps {
  readonly className?: string
}

export default TagListView
