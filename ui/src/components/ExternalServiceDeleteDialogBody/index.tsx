'use client'

import type { FunctionComponent } from 'react'
import { useCallback } from 'react'
import Button from '@mui/material/Button'
import DialogActions from '@mui/material/DialogActions'
import DialogContent from '@mui/material/DialogContent'
import DialogContentText from '@mui/material/DialogContentText'
import LoadingButton from '@mui/lab/LoadingButton'
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

  const handleClickDelete = useCallback(() => {
    deleteExternalService({ id: externalService.id }).then(
      () => {
        close()
        onDelete(externalService)
      },
      e => {
        console.error('Error deleting external service\n', e)
      },
    )
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
        <LoadingButton color="error" onClick={handleClickDelete} loading={loading}>削除</LoadingButton>
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
