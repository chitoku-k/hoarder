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
import { TAG_ATTACHING_TO_DESCENDANT, TAG_ATTACHING_TO_ITSELF, useAttachTag, useDetachTag, useError } from '@/hooks'
import type { Tag } from '@/types'

import styles from './styles.module.scss'

const TagMoveDialogBody: FunctionComponent<TagMoveDialogBodyProps> = ({
  tag,
  close,
  onMove,
}) => {
  const [ parent, setParent ] = useState(true)
  const [ destination, setDestination ] = useState<Tag | null>(tag)

  const { graphQLError } = useError()
  const [ attachTag, { error: attachError, loading: attachLoading } ] = useAttachTag()
  const [ detachTag, { error: detachError, loading: detachLoading } ] = useDetachTag()

  const handleClickMove = useCallback(() => {
    (destination
      ? attachTag({ id: tag.id, parentID: destination.id })
      : detachTag({ id: tag.id })
    ).then(
      tag => {
        close()
        onMove(tag)
      },
      (e: unknown) => {
        console.error('Error moving tag\n', e)
      },
    )
  }, [ tag, destination, attachTag, detachTag, close, onMove ])

  const disabled = useCallback(({ id }: Tag) => id === tag.id, [ tag ])

  const select = useCallback((tag: Tag | null) => {
    startTransition(() => {
      setParent(false)
      setDestination(tag)
    })
  }, [])

  const loading = attachLoading || detachLoading
  const error = attachError ?? detachError
  const tagAttachingToItself = graphQLError(attachError, TAG_ATTACHING_TO_ITSELF)
  const tagAttachingToDescendant = graphQLError(attachError, TAG_ATTACHING_TO_DESCENDANT)

  return error ? (
    <>
      <DialogContent>
        {tagAttachingToItself ? (
          <DialogContentText>
            移動元と移動先のタグが同じです
          </DialogContentText>
        ) : tagAttachingToDescendant ? (
          <DialogContentText>
            移動先のタグは移動元の子タグです
          </DialogContentText>
        ) : (
          <DialogContentText>
            タグを移動できませんでした
          </DialogContentText>
        )}
      </DialogContent>
      <DialogActions>
        <Button onClick={close} autoFocus>閉じる</Button>
      </DialogActions>
    </>
  ) : (
    <Stack className={styles.container}>
      <DialogContent>
        <DialogContentText>
          タグ「
          <Typography component="strong" fontWeight="bold">{tag.name}</Typography>
          」の移動
        </DialogContentText>
        <TagListView
          className={styles.tagList}
          initial={tag}
          readonly
          dense
          disabled={disabled}
          onSelect={select}
          selectable="column"
        />
        <Stack spacing={2} direction="row" alignItems="center">
          <Typography flexShrink={0}>移動先</Typography>
          <Card className={styles.destination}>
            <Stack className={styles.breadcrumbs} spacing={1} direction="row">
              {destination ? (
                <TagBreadcrumbsList id={destination.id} parent={parent} root />
              ) : (
                <TagBreadcrumbsList root />
              )}
            </Stack>
          </Card>
        </Stack>
      </DialogContent>
      <DialogActions>
        <Button onClick={close} autoFocus>キャンセル</Button>
        <Button onClick={handleClickMove} loading={loading} disabled={tag.id === destination?.id}>移動</Button>
      </DialogActions>
    </Stack>
  )
}

export interface TagMoveDialogBodyProps {
  tag: Tag
  close: () => void
  onMove: (tag: Tag) => void
}

export default TagMoveDialogBody
