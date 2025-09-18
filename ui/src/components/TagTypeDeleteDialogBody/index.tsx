'use client'

import type { FunctionComponent } from 'react'
import { useCallback } from 'react'
import Button from '@mui/material/Button'
import DialogActions from '@mui/material/DialogActions'
import DialogContent from '@mui/material/DialogContent'
import DialogContentText from '@mui/material/DialogContentText'
import Typography from '@mui/material/Typography'

import { useDeleteTagType } from '@/hooks'
import type { TagType } from '@/types'

const TagTypeDeleteDialogBody: FunctionComponent<TagTypeDeleteDialogBodyProps> = ({
  tagType,
  close,
  onDelete,
}) => {
  const [ deleteTagType, { error, loading } ] = useDeleteTagType()

  const handleClickDelete = useCallback(async () => {
    try {
      await deleteTagType({ id: tagType.id })
      close()
      onDelete(tagType)
    } catch (e) {
      console.error('Error deleting tag type\n', e)
    }
  }, [ deleteTagType, tagType, close, onDelete ])

  return error ? (
    <>
      <DialogContent>
        <DialogContentText>タイプを削除できませんでした</DialogContentText>
      </DialogContent>
      <DialogActions>
        <Button onClick={close} autoFocus>閉じる</Button>
      </DialogActions>
    </>
  ) : (
    <>
      <DialogContent>
        <DialogContentText>
          タイプ「
          <Typography component="strong" fontWeight="bold">{tagType.name}</Typography>
          」を削除しますか？
        </DialogContentText>
      </DialogContent>
      <DialogActions>
        <Button onClick={close} autoFocus>キャンセル</Button>
        <Button color="error" onClick={handleClickDelete} loading={loading}>削除</Button>
      </DialogActions>
    </>
  )
}

export interface TagTypeDeleteDialogBodyProps {
  readonly tagType: TagType
  readonly close: () => void
  readonly onDelete: (tagType: TagType) => void
}

export default TagTypeDeleteDialogBody
