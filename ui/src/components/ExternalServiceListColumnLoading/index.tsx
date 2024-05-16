'use client'

import type { FunctionComponent } from 'react'
import CircularProgress from '@mui/material/CircularProgress'
import Stack from '@mui/material/Stack'

import styles from './styles.module.scss'

const ExternalServiceListColumnLoading: FunctionComponent = () => (
  <Stack className={styles.column}>
    <CircularProgress />
  </Stack>
)

export default ExternalServiceListColumnLoading
