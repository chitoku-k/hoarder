'use client'

import type { FunctionComponent, Ref } from 'react'
import { useCallback, useMemo } from 'react'
import clsx from 'clsx'
import type { Components } from 'react-virtuoso'
import { Virtuoso } from 'react-virtuoso'
import ImageList from '@mui/material/ImageList'
import ImageListItem from '@mui/material/ImageListItem'
import Stack from '@mui/material/Stack'

import MediumItemImageItem from '@/components/MediumItemImageItem'
import type { Replica } from '@/types'

import styles from './styles.module.scss'

const MediumItemImageList: FunctionComponent<MediumItemImageListProps> = ({
  className,
  replicas,
  ...props
}) => {
  const computeItemKey = useCallback((_index: number, current: Replica) => current.id, [])

  const components: Components<Replica> = useMemo(() => ({
    List: ({ children, ref, ...rest }) => (
      <ImageList
        ref={ref as Ref<HTMLUListElement>}
        className={clsx(styles.imageList, className)}
        cols={1}
        {...rest}
        {...props}
      >
        {children ?? []}
      </ImageList>
    ),
    Item: ({ item, ...rest }) => (
      <ImageListItem
        className={styles.imageListItem}
        sx={{
          height: typeof item.height === 'number' && Number.isFinite(item.height)
            ? `min(100%, ${item.height.toString()}px) !important`
            : null,
        }}
        {...rest}
      />
    ),
  }), [ props, className ])

  const itemContent = useCallback((_index: number, item: Replica) => (
    <MediumItemImageItem className={styles.imageItem} replica={item} />
  ), [])

  const itemSize = useCallback((el: HTMLElement) => el.getBoundingClientRect().height, [])

  return (
    <Stack className={styles.container}>
      <Virtuoso
        className={styles.imageListContainer}
        data={replicas}
        initialItemCount={replicas.length}
        increaseViewportBy={4096}
        computeItemKey={computeItemKey}
        components={components}
        itemContent={itemContent}
        itemSize={itemSize}
        useWindowScroll
      />
    </Stack>
  )
}

export interface MediumItemImageListProps {
  className?: string
  gap?: number
  replicas: Replica[]
}

export default MediumItemImageList
