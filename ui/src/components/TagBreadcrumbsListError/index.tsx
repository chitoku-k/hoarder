'use client'

import type { FunctionComponent } from 'react'
import { useErrorBoundary } from 'react-error-boundary'
import Link from '@mui/material/Link'
import Stack from '@mui/material/Stack'
import Typography from '@mui/material/Typography'
import ErrorIcon from '@mui/icons-material/Error'

import styles from './styles.module.scss'

const TagBreadcrumbsListError: FunctionComponent = () => {
  const { resetBoundary } = useErrorBoundary()

  return (
    <Stack className={styles.breadcrumbs} direction="row" spacing={1} alignItems="center">
      <ErrorIcon className={styles.icon} fontSize="small" />
      <Stack direction="row" spacing={1} alignItems="center">
        <Typography className={styles.text}>
          タグを読み込めませんでした
        </Typography>
        <Link className={styles.retry} fontSize={14} underline="hover" onClick={resetBoundary}>
          再読み込み
        </Link>
      </Stack>
    </Stack>
  )
}

export default TagBreadcrumbsListError
