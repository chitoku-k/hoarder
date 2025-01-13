import type { FunctionComponent } from 'react'
import Container from '@mui/material/Container'

import MediumListView from '@/components/MediumListView'
import type { Source, Tag, TagType } from '@/types'

import styles from './styles.module.scss'

const MediumList: FunctionComponent<MediumListProps> = ({
  number,
  sources,
  tagTagTypes,
}) => (
  <Container className={styles.container} disableGutters>
    <MediumListView number={number} sources={sources} tagTagTypes={tagTagTypes} />
  </Container>
)

export interface MediumListProps {
  number: number
  sources?: Source[]
  tagTagTypes?: {
    tag: Tag
    type: TagType
  }[]
}

export default MediumList
