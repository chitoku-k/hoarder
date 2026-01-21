import type { FunctionComponent } from 'react'
import Button from '@mui/material/Button'
import CircularProgress from '@mui/material/CircularProgress'
import Stack from '@mui/material/Stack'

import styles from './styles.module.scss'

const MediumItemFileAppendDialogLoading: FunctionComponent<MediumItemFileAppendDialogLoadingProps> = ({
  cancel,
}) => (
  <Stack className={styles.container} spacing={2} alignItems="center" justifyContent="center">
    <CircularProgress color="inherit" />
    <Button color="inherit" onClick={cancel}>キャンセル</Button>
  </Stack>
)

export interface MediumItemFileAppendDialogLoadingProps {
  readonly cancel?: () => void
}

export default MediumItemFileAppendDialogLoading
