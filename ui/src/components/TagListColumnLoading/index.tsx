import type { FunctionComponent } from 'react'
import CircularProgress from '@mui/material/CircularProgress'
import Stack from '@mui/material/Stack'

import styles from './styles.module.scss'

const TagListColumnLoading: FunctionComponent = () => (
  <Stack className={styles.column}>
    <CircularProgress />
  </Stack>
)

export default TagListColumnLoading
