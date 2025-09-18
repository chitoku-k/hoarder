'use client'

import type { FunctionComponent, ReactNode } from 'react'

import { useTagsChildren } from '@/hooks'
import type { Tag } from '@/types'

const TagListColumnBodyListChildren: FunctionComponent<TagListColumnBodyListChildrenProps> = ({
  id,
  component,
}) => {
  const children = useTagsChildren({ ids: [ id ] })
  if (!children) {
    return null
  }

  return (
    <>
      {component(children)}
    </>
  )
}

export interface TagListColumnBodyListChildrenProps {
  readonly id: string
  readonly component: (tags: readonly Tag[]) => ReactNode
}

export default TagListColumnBodyListChildren
