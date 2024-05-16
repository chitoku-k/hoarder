'use client'

import type { FunctionComponent } from 'react'
import { useErrorBoundary } from 'react-error-boundary'
import Link from '@mui/material/Link'
import Stack from '@mui/material/Stack'
import Typography from '@mui/material/Typography'
import SellIcon from '@mui/icons-material/Sell'

import styles from './styles.module.scss'

const TagListViewError: FunctionComponent = () => {
  const { resetBoundary } = useErrorBoundary()

  return (
    <Stack flexGrow={1} alignItems="center" justifyContent="center" spacing={2}>
      <SellIcon className={styles.icon} />
      <Typography className={styles.text}>
        タグを読み込めませんでした
      </Typography>
      <Link className={styles.retry} fontSize={14} underline="hover" onClick={resetBoundary}>
        再読み込み
      </Link>
    </Stack>
  )
}

export default TagListViewError
