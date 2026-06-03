'use client'

import type { FunctionComponent } from 'react'
import { useErrorBoundary } from 'react-error-boundary'
import IconButton from '@mui/material/IconButton'
import Stack from '@mui/material/Stack'
import Typography from '@mui/material/Typography'
import ErrorIcon from '@mui/icons-material/Error'
import RefreshIcon from '@mui/icons-material/Refresh'

import styles from './styles.module.scss'

const AutocompleteExternalServiceError: FunctionComponent = () => {
  const { resetBoundary } = useErrorBoundary()

  return (
    <Stack className={styles.container} direction="row" spacing={1}>
      <ErrorIcon className={styles.icon} fontSize="small" />
      <Stack className={styles.body} direction="row">
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

export default AutocompleteExternalServiceError
