import type { FunctionComponent } from 'react'
import Link from 'next/link'
import ImageListItem from '@mui/material/ImageListItem'

import MediumListItemThumbnail from '@/components/MediumListItemThumbnail'
import MediumListItemCount from '@/components/MediumListItemCount'
import type { Medium } from '@/types'

import styles from './styles.module.scss'

export const MediumListItem: FunctionComponent<MediumListItemProps> = ({
  medium,
  size,
}) => (
  <ImageListItem className={styles.item}>
    <Link className={styles.link} href={`/media/${medium.id}`}>
      <MediumListItemThumbnail replica={medium.replicas?.[0]} size={size} />
      <MediumListItemCount className={styles.count} count={medium.replicas?.length} fontSize="small" />
    </Link>
  </ImageListItem>
)

export interface MediumListItemProps {
  readonly medium: Medium
  readonly size: number
}

export default MediumListItem
