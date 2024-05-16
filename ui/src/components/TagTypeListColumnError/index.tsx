'use client'

import type { FunctionComponent } from 'react'
import { useErrorBoundary } from 'react-error-boundary'
import Link from '@mui/material/Link'
import Stack from '@mui/material/Stack'
import Typography from '@mui/material/Typography'
import LabelIcon from '@mui/icons-material/Label'

import styles from './styles.module.scss'

const TagTypeListColumnError: FunctionComponent = () => {
  const { resetBoundary } = useErrorBoundary()

  return (
    <Stack className={styles.column} spacing={2}>
      <LabelIcon className={styles.icon} />
      <Typography className={styles.text}>
        タイプを読み込めませんでした
      </Typography>
      <Link className={styles.retry} fontSize={14} underline="hover" onClick={resetBoundary}>
        再読み込み
      </Link>
    </Stack>
  )
}

export default TagTypeListColumnError
