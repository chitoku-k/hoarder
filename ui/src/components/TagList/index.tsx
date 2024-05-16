import type { FunctionComponent } from 'react'
import Container from '@mui/material/Container'
import Stack from '@mui/material/Stack'
import Typography from '@mui/material/Typography'

import TagListView from '@/components/TagListView'
import TagTypeListView from '@/components/TagTypeListView'

import styles from './styles.module.scss'

const TagList: FunctionComponent = () => (
  <Container className={styles.container}>
    <Stack spacing={4}>
      <Stack>
        <Typography className={styles.header} variant="h2">
          タグ
        </Typography>
        <TagListView className={styles.tagList} />
      </Stack>
      <Stack>
        <Typography className={styles.header} variant="h2">
          タイプ
        </Typography>
        <TagTypeListView className={styles.tagTypeList} />
      </Stack>
    </Stack>
  </Container>
)

export default TagList
