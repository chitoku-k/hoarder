'use client'

import type { FunctionComponent } from 'react'
import { useCallback, useTransition } from 'react'
import ImageList from '@mui/material/ImageList'
import LoadingButton from '@mui/lab/LoadingButton'
import Stack from '@mui/material/Stack'
import ExpandMoreIcon from '@mui/icons-material/ExpandMore'

import MediumListItem from '@/components/MediumListItem'
import { useMedia } from '@/hooks'

import styles from './styles.module.scss'

const MediumListViewBody: FunctionComponent<MediumListViewBodyProps> = ({
  number,
}) => {
  const [ loading, startTransition ] = useTransition()
  const [ media, hasNextPage, fetchMore ] = useMedia(number)

  const handleClickMore = useCallback(() => {
    startTransition(() => {
      fetchMore()
    })
  }, [ fetchMore ])

  return (
    <Stack>
      <ImageList className={styles.container} gap={40}>
        {media.map(medium => (
          <MediumListItem key={medium.id} medium={medium} size={160} />
        ))}
      </ImageList>
      <Stack className={styles.pagination} alignItems="center">
        {hasNextPage ? (
          <LoadingButton
            variant="outlined"
            loading={loading}
            loadingPosition="end"
            endIcon={<ExpandMoreIcon />}
            onClick={handleClickMore}
          >
            次へ
          </LoadingButton>
        ) : null}
      </Stack>
    </Stack>
  )
}

export interface MediumListViewBodyProps {
  number: number
}

export default MediumListViewBody
