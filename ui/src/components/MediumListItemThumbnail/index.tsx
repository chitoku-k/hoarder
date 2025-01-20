'use client'

import type { FunctionComponent } from 'react'
import Image from 'next/image'
import Stack from '@mui/material/Stack'
import PhotoOutlinedIcon from '@mui/icons-material/PhotoOutlined'

import type { Replica } from '@/types'

import styles from './styles.module.scss'

const MediumListItemThumbnail: FunctionComponent<MediumListItemThumbnailProps> = ({
  replica,
  size,
}) => replica?.thumbnail && typeof replica.width === 'number' && typeof replica.height === 'number' ? (
  <Image
    className={styles.image}
    src={replica.thumbnail.url}
    width={replica.width >= replica.height ? size : Math.round(size / (replica.height / replica.width))}
    height={replica.width <= replica.height ? size : Math.round(size / (replica.width / replica.height))}
    quality={100}
    loading="lazy"
    alt=""
  />
) : (
  <Stack className={styles.noimage} alignItems="center" justifyContent="center">
    <PhotoOutlinedIcon className={styles.noimageIcon} />
  </Stack>
)

export interface MediumListItemThumbnailProps {
  replica?: Replica
  size: number
}

export default MediumListItemThumbnail
