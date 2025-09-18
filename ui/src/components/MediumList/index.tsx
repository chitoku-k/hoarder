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
  readonly number: number
  readonly sources?: readonly Source[]
  readonly tagTagTypes?: readonly {
    readonly tag: Tag
    readonly type: TagType
  }[]
}

export default MediumList
