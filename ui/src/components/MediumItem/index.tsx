import type { FunctionComponent } from 'react'
import Container from '@mui/material/Container'

import MediumItemView from '@/components/MediumItemView'

import styles from './styles.module.scss'

const MediumItem: FunctionComponent<MediumItemProps> = ({
  id,
}) => (
  <Container className={styles.container}>
    <MediumItemView id={id} />
  </Container>
)

export interface MediumItemProps {
  readonly id: string
}

export default MediumItem
