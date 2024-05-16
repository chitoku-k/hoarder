'use client'

import type { FunctionComponent } from 'react'
import CircularProgress from '@mui/material/CircularProgress'
import Stack from '@mui/material/Stack'

import styles from './styles.module.scss'

const TagSelectDialogLoading: FunctionComponent = () => (
  <Stack className={styles.container} alignItems="center" justifyContent="center">
    <CircularProgress color="inherit" />
  </Stack>
)

export default TagSelectDialogLoading
