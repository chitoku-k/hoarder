'use client'

import type { FunctionComponent } from 'react'
import { use, useCallback } from 'react'
import Button from '@mui/material/Button'
import DialogActions from '@mui/material/DialogActions'
import DialogContent from '@mui/material/DialogContent'
import DialogContentText from '@mui/material/DialogContentText'

const MediumItemFileAppendDialogBody: FunctionComponent<MediumItemFileAppendDialogBodyProps> = ({
  entries,
  close,
  onAppend,
}) => {
  const files = use(entries)

  const handleClickClose = useCallback(() => {
    close()
  }, [ close ])

  const handleClickAppend = useCallback(() => {
    onAppend?.(files)
    close()
  }, [ onAppend, close, files ])

  return files.length ? (
    <>
      <DialogContent>
        <DialogContentText>{files.length} 件の項目を追加しますか？</DialogContentText>
      </DialogContent>
      <DialogActions>
        <Button onClick={handleClickClose}>キャンセル</Button>
        <Button onClick={handleClickAppend} autoFocus>追加</Button>
      </DialogActions>
    </>
  ) : null
}

export type Folder = (File | Folder)[]

export interface MediumItemFileAppendDialogBodyProps {
  entries: Promise<File[]>
  close: () => void
  onAppend: (files: File[]) => void
}

export default MediumItemFileAppendDialogBody
