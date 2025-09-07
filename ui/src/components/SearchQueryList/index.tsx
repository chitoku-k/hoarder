'use client'

import type { FunctionComponent } from 'react'
import Chip from '@mui/material/Chip'
import Stack from '@mui/material/Stack'
import Toolbar from '@mui/material/Toolbar'
import LabelIcon from '@mui/icons-material/Label'

import SourceURL from '@/components/SourceURL'
import TagBreadcrumbsList from '@/components/TagBreadcrumbsList'
import { useSearch } from '@/contexts'
import type { Source, Tag, TagType } from '@/types'

import styles from './styles.module.scss'

const SearchQueryList: FunctionComponent<SearchQueryListProps> = ({
  sources,
  tagTagTypes,
}) => {
  const { removeQuery } = useSearch()

  return (
    <Toolbar className={styles.search}>
      <Stack className={styles.queries} direction="row" spacing={1} flexWrap="wrap" useFlexGap>
        {sources?.map(source => (
          <Chip
            key={source.id}
            label={<SourceURL source={source} noLink />}
            onDelete={() => removeQuery({ sourceID: source.id })}
          />
        ))}
        {tagTagTypes?.map(({ type, tag }) => (
          <Chip
            key={`${type.id}:${tag.id}`}
            label={
              <Stack direction="row" spacing={1} alignItems="start">
                <Stack direction="row" alignItems="start">
                  <LabelIcon className={styles.tagTypeIcon} fontSize="small" />
                  <span className={styles.tagTypeText}>{type.name}:</span>
                </Stack>
                <TagBreadcrumbsList tag={tag} />
              </Stack>
            }
            onDelete={() => removeQuery({ tagTagTypeIDs: [ { tagID: tag.id, typeID: type.id } ] })}
          />
        ))}
      </Stack>
    </Toolbar>
  )
}

export interface SearchQueryListProps {
  sources?: Source[]
  tagTagTypes?: {
    tag: Tag
    type: TagType
  }[]
}

export default SearchQueryList
