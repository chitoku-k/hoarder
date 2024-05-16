'use client'

import { useCallback, type FunctionComponent } from 'react'
import Button from '@mui/material/Button'
import CircularProgress from '@mui/material/CircularProgress'
import Stack from '@mui/material/Stack'

import styles from './styles.module.scss'

const MediumItemFileAppendDialogLoading: FunctionComponent<MediumItemFileAppendDialogLoadingProps> = ({
  cancel,
}) => {
  const handleClickCancel = useCallback(() => {
    cancel?.()
  }, [ cancel ])

  return (
    <Stack className={styles.container} spacing={2} alignItems="center" justifyContent="center">
      <CircularProgress color="inherit" />
      <Button color="inherit" onClick={handleClickCancel}>キャンセル</Button>
    </Stack>
  )
}

export interface MediumItemFileAppendDialogLoadingProps {
  cancel?: () => void
}

export default MediumItemFileAppendDialogLoading
