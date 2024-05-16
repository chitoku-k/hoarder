'use client'

import type { FunctionComponent } from 'react'
import { useCallback } from 'react'
import Button from '@mui/material/Button'
import DialogActions from '@mui/material/DialogActions'
import DialogContent from '@mui/material/DialogContent'
import DialogContentText from '@mui/material/DialogContentText'
import LoadingButton from '@mui/lab/LoadingButton'

import { useDeleteMedium } from '@/hooks'
import type { Medium } from '@/types'

const MediumDeleteDialogBody: FunctionComponent<MediumDeleteDialogBodyProps> = ({
  medium,
  close,
  onDelete,
}) => {
  const [ deleteMedium, { error, loading } ] = useDeleteMedium()

  const handleClickDelete = useCallback(() => {
    deleteMedium({ id: medium.id }).then(
      () => {
        close()
        onDelete(medium)
      },
      e => {
        console.error('Error deleting medium\n', e)
      },
    )
  }, [ deleteMedium, medium, close, onDelete ])

  return error ? (
    <>
      <DialogContent>
        <DialogContentText>メディアを削除できませんでした</DialogContentText>
      </DialogContent>
      <DialogActions>
        <Button onClick={close} autoFocus>閉じる</Button>
      </DialogActions>
    </>
  ) : (
    <>
      <DialogContent>
        <DialogContentText>
          メディアを削除しますか？
        </DialogContentText>
      </DialogContent>
      <DialogActions>
        <Button onClick={close} autoFocus>キャンセル</Button>
        <LoadingButton color="error" onClick={handleClickDelete} loading={loading}>削除</LoadingButton>
      </DialogActions>
    </>
  )
}

export interface MediumDeleteDialogBodyProps {
  medium: Medium
  close: () => void
  onDelete: (medium: Medium) => void
}

export default MediumDeleteDialogBody
