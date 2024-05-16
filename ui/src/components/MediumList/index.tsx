import type { FunctionComponent } from 'react'
import Container from '@mui/material/Container'

import MediumListView from '@/components/MediumListView'

const MediumList: FunctionComponent<MediumListProps> = ({
  number,
}) => (
  <Container disableGutters>
    <MediumListView number={number} />
  </Container>
)

export interface MediumListProps {
  number: number
}

export default MediumList
