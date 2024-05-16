'use client'

import type { FunctionComponent } from 'react'
import CircularProgress from '@mui/material/CircularProgress'
import Stack from '@mui/material/Stack'
import NavigateNextIcon from '@mui/icons-material/NavigateNext'
import SellIcon from '@mui/icons-material/Sell'

import styles from './styles.module.scss'

const TagBreadcrumbsListLoading: FunctionComponent = () => (
  <Stack className={styles.breadcrumbs} direction="row" alignItems="top">
    <SellIcon className={styles.icon} fontSize="small" />
    <NavigateNextIcon className={styles.nextIcon} fontSize="small" />
    <CircularProgress className={styles.icon} size={20} />
  </Stack>
)

export default TagBreadcrumbsListLoading
