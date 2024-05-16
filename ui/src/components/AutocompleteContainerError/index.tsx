'use client'

import type { FunctionComponent } from 'react'
import { useErrorBoundary } from 'react-error-boundary'
import IconButton from '@mui/material/IconButton'
import Stack from '@mui/material/Stack'
import Typography from '@mui/material/Typography'
import FolderOffIcon from '@mui/icons-material/FolderOff'
import RefreshIcon from '@mui/icons-material/Refresh'

import styles from './styles.module.scss'

const AutocompleteContainerError: FunctionComponent = () => {
  const { resetBoundary } = useErrorBoundary()

  return (
    <Stack className={styles.container} direction="row" spacing={1} flexGrow={1} alignItems="center">
      <FolderOffIcon className={styles.icon} fontSize="small" />
      <Stack direction="row" flexGrow={1} alignItems="center" justifyContent="space-between">
        <Typography className={styles.text}>
          読み込めませんでした
        </Typography>
        <IconButton size="small" onClick={resetBoundary}>
          <RefreshIcon fontSize="inherit" />
        </IconButton>
      </Stack>
    </Stack>
  )
}

export default AutocompleteContainerError
