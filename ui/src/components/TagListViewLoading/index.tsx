import type { FunctionComponent } from 'react'
import CircularProgress from '@mui/material/CircularProgress'
import Stack from '@mui/material/Stack'

import styles from './styles.module.scss'

const TagListViewLoading: FunctionComponent = () => (
  <Stack className={styles.container}>
    <CircularProgress size={48} />
  </Stack>
)

export default TagListViewLoading
