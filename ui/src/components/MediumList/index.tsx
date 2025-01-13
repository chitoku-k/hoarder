import type { FunctionComponent } from 'react'
import Container from '@mui/material/Container'

import MediumListView from '@/components/MediumListView'

import styles from './styles.module.scss'

const MediumList: FunctionComponent<MediumListProps> = ({
  number,
}) => (
  <Container className={styles.container} disableGutters>
    <MediumListView number={number} />
  </Container>
)

export interface MediumListProps {
  number: number
}

export default MediumList
