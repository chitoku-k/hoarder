'use client'

import type { FunctionComponent, ReactNode } from 'react'

import { useTagsRoot } from '@/hooks'
import type { Tag } from '@/types'

const TagListColumnBodyListRoot: FunctionComponent<TagListColumnBodyListRootProps> = ({
  number,
  component,
}) => {
  const root = useTagsRoot({ number })
  if (!root) {
    return null
  }

  const [ children, hasNextPage, fetchMore ] = root
  return (
    <>
      {component(children, hasNextPage, fetchMore)}
    </>
  )
}

export interface TagListColumnBodyListRootProps {
  number: number
  component: (tags: Tag[], hasNextPage: boolean, fetchMore: () => Promise<void>) => ReactNode
}

export default TagListColumnBodyListRoot
