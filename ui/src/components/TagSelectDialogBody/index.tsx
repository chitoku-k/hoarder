'use client'

import type { FunctionComponent } from 'react'
import { useState, useCallback, startTransition } from 'react'
import Button from '@mui/material/Button'
import Card from '@mui/material/Card'
import DialogActions from '@mui/material/DialogActions'
import DialogContent from '@mui/material/DialogContent'
import DialogContentText from '@mui/material/DialogContentText'
import Stack from '@mui/material/Stack'
import Typography from '@mui/material/Typography'

import TagBreadcrumbsList from '@/components/TagBreadcrumbsList'
import TagListView from '@/components/TagListView'
import type { Tag } from '@/types'

import styles from './styles.module.scss'

const TagSelectDialogBody: FunctionComponent<TagSelectDialogBodyProps> = ({
  close,
  onSelect,
}) => {
  const [ tag, setTag ] = useState<Tag | null>(null)

  const handleClickSelect = useCallback(() => {
    if (!tag) {
      return
    }

    close()
    onSelect(tag)
  }, [ close, onSelect, tag ])

  const select = useCallback((tag: Tag | null) => {
    startTransition(() => {
      setTag(tag)
    })
  }, [])

  return (
    <Stack className={styles.container}>
      <DialogContent>
        <DialogContentText>
          タグの選択
        </DialogContentText>
        <TagListView
          className={styles.tagList}
          dense
          onSelect={select}
          selectable="tag"
        />
        <Stack spacing={2} direction="row" alignItems="center">
          <Typography flexShrink={0}>タグ</Typography>
          <Card className={styles.destination}>
            <Stack className={styles.breadcrumbs} spacing={1} direction="row">
              {tag ? (
                <TagBreadcrumbsList id={tag.id} />
              ) : (
                <TagBreadcrumbsList root />
              )}
            </Stack>
          </Card>
        </Stack>
      </DialogContent>
      <DialogActions>
        <Button onClick={close} autoFocus>キャンセル</Button>
        <Button onClick={handleClickSelect} disabled={!tag}>選択</Button>
      </DialogActions>
    </Stack>
  )
}

export interface TagSelectDialogBodyProps {
  close: () => void
  onSelect: (tag: Tag) => void
}

export default TagSelectDialogBody
