import type { FunctionComponent } from 'react'
import { Suspense } from 'react'
import { ErrorBoundary } from 'react-error-boundary'

import type { TagBreadcrumbsListBodyProps } from '@/components/TagBreadcrumbsListBody'
import TagBreadcrumbsListBody from '@/components/TagBreadcrumbsListBody'
import TagBreadcrumbsListError from '@/components/TagBreadcrumbsListError'
import TagBreadcrumbsListLoading from '@/components/TagBreadcrumbsListLoading'

const TagBreadcrumbsList: FunctionComponent<TagBreadcrumbsListProps> = ({ ...props }) => (
  <ErrorBoundary fallback={<TagBreadcrumbsListError />}>
    <Suspense fallback={<TagBreadcrumbsListLoading />}>
      <TagBreadcrumbsListBody {...props} />
    </Suspense>
  </ErrorBoundary>
)

export type TagBreadcrumbsListProps = TagBreadcrumbsListBodyProps

export default TagBreadcrumbsList
