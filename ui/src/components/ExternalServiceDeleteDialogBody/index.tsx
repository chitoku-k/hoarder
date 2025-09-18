'use client'

import type { FunctionComponent } from 'react'
import { useCallback } from 'react'
import Button from '@mui/material/Button'
import DialogActions from '@mui/material/DialogActions'
import DialogContent from '@mui/material/DialogContent'
import DialogContentText from '@mui/material/DialogContentText'
import Typography from '@mui/material/Typography'

import { useDeleteExternalService } from '@/hooks'
import type { ExternalService } from '@/types'

const ExternalServiceDeleteDialogBody: FunctionComponent<ExternalServiceDeleteDialogBodyProps> = ({
  externalService,
  close,
  onDelete,
}) => {
  const [ deleteExternalService, { error, loading } ] = useDeleteExternalService()

  const handleClickClose = useCallback(() => {
    close()
  }, [ close ])

  const handleClickDelete = useCallback(async () => {
    try {
      await deleteExternalService({ id: externalService.id })
      close()
      onDelete(externalService)
    } catch (e) {
      console.error('Error deleting external service\n', e)
    }
  }, [ deleteExternalService, externalService, close, onDelete ])

  return error ? (
    <>
      <DialogContent>
        <DialogContentText>サービスを削除できませんでした</DialogContentText>
      </DialogContent>
      <DialogActions>
        <Button onClick={handleClickClose} autoFocus>閉じる</Button>
      </DialogActions>
    </>
  ) : (
    <>
      <DialogContent>
        <DialogContentText>
          サービス「
          <Typography component="strong" fontWeight="bold">{externalService.name}</Typography>
          」を削除しますか？
        </DialogContentText>
      </DialogContent>
      <DialogActions>
        <Button onClick={handleClickClose} autoFocus>キャンセル</Button>
        <Button color="error" onClick={handleClickDelete} loading={loading}>削除</Button>
      </DialogActions>
    </>
  )
}

export interface ExternalServiceDeleteDialogBodyProps {
  externalService: ExternalService
  close: () => void
  onDelete: (externalService: ExternalService) => void
}

export default ExternalServiceDeleteDialogBody
