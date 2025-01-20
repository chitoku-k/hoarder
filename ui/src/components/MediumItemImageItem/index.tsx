'use client'

import type { CSSProperties, FunctionComponent, ReactNode } from 'react'
import { memo } from 'react'
import clsx from 'clsx'
import Stack from '@mui/material/Stack'
import PhotoIcon from '@mui/icons-material/Photo'

import Image from '@/components/Image'
import ImageBodyBlob from '@/components/ImageBodyBlob'
import ImageBodyNext from '@/components/ImageBodyNext'
import type { ReplicaCreate } from '@/components/MediumItemImageEdit'
import { isReplica } from '@/components/MediumItemImageEdit'
import type { Replica } from '@/types'

import styles from './styles.module.scss'

const MediumItemImageItem: FunctionComponent<MediumItemImageItemProps> = ({
  className,
  replica,
  children,
}) => {
  const aspectRatio = replica.width && replica.height
    ? `${replica.width} / ${replica.height}`
    : undefined

  const style: CSSProperties = { aspectRatio }

  return typeof replica.width === 'number' && typeof replica.height === 'number' ? (
    <Image
      className={clsx(styles.item, className)}
      width={replica.width}
      height={replica.height}
      style={style}
    >
      <Stack className={clsx(styles.wrapper, className)} alignItems="stretch" justifyContent="stretch" style={style}>
        {isReplica(replica) ? replica.url ? (
          <ImageBodyNext
            className={styles.image}
            src={replica.url}
            width={replica.width}
            height={replica.height}
            priority
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
        {children}
      </Stack>
    </Image>
  ) : null
}

export interface MediumItemImageItemProps {
  className?: string
  replica: Replica | ReplicaCreate
  fixed?: boolean
  children?: ReactNode
}

export default memo(MediumItemImageItem)
