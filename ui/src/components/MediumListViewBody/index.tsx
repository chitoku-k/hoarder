'use client'

import type { FunctionComponent } from 'react'
import { useCallback, useTransition } from 'react'
import ImageList from '@mui/material/ImageList'
import LoadingButton from '@mui/lab/LoadingButton'
import Stack from '@mui/material/Stack'
import Typography from '@mui/material/Typography'
import CollectionsIcon from '@mui/icons-material/Collections'
import ExpandMoreIcon from '@mui/icons-material/ExpandMore'

import MediumListItem from '@/components/MediumListItem'
import { useMedia } from '@/hooks'
import type { Source, Tag, TagType } from '@/types'

import styles from './styles.module.scss'

const MediumListViewBody: FunctionComponent<MediumListViewBodyProps> = ({
  number,
  sources,
  tagTagTypes,
}) => {
  const [ loading, startTransition ] = useTransition()

  const sourceIDs = sources?.map(({ id }) => id)
  const tagTagTypeIDs = tagTagTypes?.map(({ tag, type }) => ({ tagID: tag.id, typeID: type.id }))

  const [ media, hasNextPage, fetchMore ] = useMedia(number, { sourceIDs, tagTagTypeIDs })

  const handleClickMore = useCallback(() => {
    startTransition(() => {
      fetchMore()
    })
  }, [ fetchMore ])

  return media.length ? (
    <Stack flexGrow={1}>
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
  ) : (
    <Stack className={styles.noMedia} flexGrow={1} alignItems="center" justifyContent="center" spacing={2}>
      <CollectionsIcon className={styles.icon} />
      <Typography className={styles.text}>
        メディアがありません
      </Typography>
    </Stack>
  )
}

export interface MediumListViewBodyProps {
  number: number
  sources?: Source[]
  tagTagTypes?: {
    tag: Tag
    type: TagType
  }[]
}

export default MediumListViewBody
