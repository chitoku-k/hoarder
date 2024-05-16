import type { FunctionComponent } from 'react'
import Container from '@mui/material/Container'

import MediumItemView from '@/components/MediumItemView'

const MediumItem: FunctionComponent<MediumItemProps> = ({
  id,
}) => (
  <Container>
    <MediumItemView id={id} />
  </Container>
)

export interface MediumItemProps {
  id: string
}

export default MediumItem
