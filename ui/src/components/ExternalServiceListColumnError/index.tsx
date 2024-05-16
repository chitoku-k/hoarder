'use client'

import type { FunctionComponent } from 'react'
import { useErrorBoundary } from 'react-error-boundary'
import Link from '@mui/material/Link'
import Stack from '@mui/material/Stack'
import Typography from '@mui/material/Typography'
import FolderSpecialIcon from '@mui/icons-material/FolderSpecial'

import styles from './styles.module.scss'

const ExternalServiceListColumnError: FunctionComponent = () => {
  const { resetBoundary } = useErrorBoundary()

  return (
    <Stack className={styles.column} spacing={2}>
      <FolderSpecialIcon className={styles.icon} />
      <Typography className={styles.text}>
        サービスを読み込めませんでした
      </Typography>
      <Link className={styles.retry} fontSize={14} underline="hover" onClick={resetBoundary}>
        再読み込み
      </Link>
    </Stack>
  )
}

export default ExternalServiceListColumnError
