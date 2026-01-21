import type { FunctionComponent } from 'react'
import Button from '@mui/material/Button'
import Stack from '@mui/material/Stack'
import Typography from '@mui/material/Typography'

import MediumItemMetadataHeader from '@/components/MediumItemMetadataHeader'
import TagBreadcrumbsList from '@/components/TagBreadcrumbsList'
import type { Medium, Tag, TagType } from '@/types'

import styles from './styles.module.scss'

const MediumItemMetadataTagList: FunctionComponent<MediumItemMetadataTagListProps> = ({
  medium,
  edit,
}) => {
  const tags = medium.tags ?? []
  const groups: readonly ReadonlyTagGroup[] = tags.reduce<TagGroup[]>((groups, { tag, type }) => {
    const group = groups.find(t => t.type.id === type.id)
    if (group) {
      group.tags.push(tag)
    } else {
      groups.push({
        type,
        tags: [ tag ],
      })
    }
    return groups
  }, [])

  return (
    <Stack>
      <MediumItemMetadataHeader title="タグ">
        <Button onClick={edit}>編集</Button>
      </MediumItemMetadataHeader>
      <Stack spacing={4}>
        {groups.length ? groups.map(({ type, tags }) => (
          <Stack key={type.id}>
            <Stack className={styles.header} direction="row" alignItems="center">
              <Typography className={styles.title} variant="h4">{type.name}</Typography>
            </Stack>
            <Stack spacing={0.5}>
              {tags.map(tag => (
                <Stack key={tag.id} direction="row" alignItems="center">
                  <TagBreadcrumbsList tag={tag} />
                  <Stack className={styles.buttonArea} />
                </Stack>
              ))}
            </Stack>
          </Stack>
        )) : (
          <Stack>
            未分類
          </Stack>
        )}
      </Stack>
    </Stack>
  )
}

export interface MediumItemMetadataTagListProps {
  readonly medium: Medium
  readonly edit: () => void
}

interface ReadonlyTagGroup {
  readonly type: TagType
  readonly tags: readonly Tag[]
}

interface TagGroup {
  type: TagType
  tags: Tag[]
}

export default MediumItemMetadataTagList
