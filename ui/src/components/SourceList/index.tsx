import type { FunctionComponent } from 'react'
import Container from '@mui/material/Container'
import Typography from '@mui/material/Typography'

import ExternalServiceListView from '@/components/ExternalServiceListView'

import styles from './styles.module.scss'

const SourceList: FunctionComponent = () => (
  <Container className={styles.container}>
    <Typography className={styles.header} variant="h2">
      サービス
    </Typography>
    <ExternalServiceListView className={styles.externalServiceList} />
  </Container>
)

export default SourceList
