'use client'

import type { FunctionComponent } from 'react'
import { useCallback } from 'react'
import Button from '@mui/material/Button'
import DialogActions from '@mui/material/DialogActions'
import DialogContent from '@mui/material/DialogContent'
import DialogContentText from '@mui/material/DialogContentText'
import Stack from '@mui/material/Stack'
import Typography from '@mui/material/Typography'
import InsertDriveFileOutlinedIcon from '@mui/icons-material/InsertDriveFileOutlined'

import styles from './styles.module.scss'

const MediumItemFileErrorDialogBody: FunctionComponent<MediumItemFileErrorDialogBodyProps> = ({
  files,
  error,
  close,
}) => {
  const handleClickClose = useCallback(() => {
    close()
  }, [ close ])

  return (
    <>
      <DialogContent>
        {error ? (
          <DialogContentText>項目が追加できませんでした</DialogContentText>
        ) : files ? (
          <>
            <DialogContentText>{files.length} 件の項目が追加できませんでした</DialogContentText>
            <ul className={styles.files}>
              {files.map((file, i) => (
                <li key={i} className={styles.file}>
                  <Stack spacing={0.5} direction="row">
                    <InsertDriveFileOutlinedIcon />
                    <Typography>{file.name}</Typography>
                  </Stack>
                </li>
              ))}
            </ul>
          </>
        ) : null}
      </DialogContent>
      <DialogActions>
        <Button onClick={handleClickClose} autoFocus>閉じる</Button>
      </DialogActions>
    </>
  )
}

export interface MediumItemFileErrorDialogBodyProps {
  readonly files?: readonly File[]
  readonly error?: unknown
  readonly close: () => void
}

export default MediumItemFileErrorDialogBody
