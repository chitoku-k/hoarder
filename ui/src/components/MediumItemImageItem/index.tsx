'use client'

import type { CSSProperties, FunctionComponent, ReactNode } from 'react'
import { memo } from 'react'
import clsx from 'clsx'
import Stack from '@mui/material/Stack'
import PhotoIcon from '@mui/icons-material/Photo'

import Image from '@/components/Image'
import ImageBodyBlob from '@/components/ImageBodyBlob'
import ImageBodyNext from '@/components/ImageBodyNext'
import ImageError from '@/components/ImageError'
import ImageLoading from '@/components/ImageLoading'
import type { ReplicaCreate } from '@/components/MediumItemImageEdit'
import { isReplica } from '@/components/MediumItemImageEdit'
import type { Replica } from '@/types'

import styles from './styles.module.scss'

const MediumItemImageItem: FunctionComponent<MediumItemImageItemProps> = ({
  className,
  replica,
  children,
}) => {
  const aspectRatio = typeof replica.width === 'number' && Number.isFinite(replica.width) && typeof replica.height === 'number' && Number.isFinite(replica.height)
    ? `${replica.width.toString()} / ${replica.height.toString()}`
    : undefined

  const style = { aspectRatio } satisfies CSSProperties
  const phase = isReplica(replica) ? replica.status.phase : 'READY'

  return (
    <Stack className={clsx(styles.wrapper, className)} alignItems="stretch" justifyContent="stretch" style={style}>
      {phase === 'READY' && typeof replica.width === 'number' && typeof replica.height === 'number' ? (
        <Image
          className={styles.item}
          width={replica.width}
          height={replica.height}
          style={style}
        >
          {isReplica(replica) ? replica.url ? (
            <ImageBodyNext
              className={styles.image}
              src={replica.url}
              width={replica.width}
              height={replica.height}
              preload
              unoptimized
              alt=""
            />
          ) : (
            <Stack className={styles.noimage} alignItems="center" justifyContent="center">
              <PhotoIcon className={styles.noimageIcon} />
            </Stack>
          ) : (
            <ImageBodyBlob
              className={styles.image}
              src={replica.blob}
              width={replica.width}
              height={replica.height}
              alt=""
            />
          )}
        </Image>
      ) : phase === 'PROCESSING' ? (
        <ImageLoading className={styles.loading} />
      ) : phase === 'ERROR' ? (
        <ImageError className={styles.error} />
      ) : null}
      {children}
    </Stack>
  )
}

export interface MediumItemImageItemProps {
  readonly className?: string
  readonly replica: Replica | ReplicaCreate
  readonly fixed?: boolean
  readonly children?: ReactNode
}

export default memo(MediumItemImageItem)
