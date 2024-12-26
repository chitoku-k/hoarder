'use client'

import type { ComponentPropsWithoutRef, FunctionComponent } from 'react'
import { useCallback } from 'react'
import IconButton from '@mui/material/IconButton'
import Stack from '@mui/material/Stack'
import Tooltip from '@mui/material/Tooltip'
import Typography from '@mui/material/Typography'
import AddCircleOutlineIcon from '@mui/icons-material/AddCircleOutline'
import RemoveCircleOutlineIcon from '@mui/icons-material/RemoveCircleOutline'
import SellIcon from '@mui/icons-material/Sell'

import AutocompleteTag from '@/components/AutocompleteTag'
import TagBreadcrumbsList from '@/components/TagBreadcrumbsList'
import type { Tag, TagType } from '@/types'

import styles from './styles.module.scss'

const MediumItemMetadataTagGroupEdit: FunctionComponent<MediumItemMetadataTagGroupEditProps> = ({
  loading,
  type,
  tags,
  focus,
  removingTagType,
  removeTagType,
  restoreTagType,
  addingTags,
  removingTags,
  addTag,
  removeTag,
  restoreTag,
}) => {
  const handleChangeNewTag = useCallback((tag: Tag | null) => {
    if (!tag) {
      return
    }

    restoreTagType(type)
    addTag(type, tag)
  }, [ restoreTagType, type, addTag ])

  const handleClickRemoveTagType = useCallback(() => {
    removeTagType(type)

    for (const tag of tags) {
      removeTag(type, tag)
    }

    for (const tag of addingTags) {
      removeTag(type, tag)
    }
  }, [ removeTagType, type, removeTag, tags, addingTags ])

  const handleClickRestoreTagType = useCallback(() => {
    restoreTagType(type)

    for (const tag of tags) {
      restoreTag(type, tag)
    }
  }, [ restoreTagType, restoreTag, type, tags ])

  const handleClickRemoveTag = (tag: Tag) => {
    removeTag(type, tag)
  }

  const handleClickRestoreTag = (tag: Tag) => {
    restoreTagType(type)
    restoreTag(type, tag)
  }

  const renderTagOption = useCallback(({ key, ...props }: ComponentPropsWithoutRef<'li'>, option: Tag) => (
    <li key={key} {...props}>
      <TagBreadcrumbsList tag={option} />
    </li>
  ), [])

  const allTags = [ ...tags, ...addingTags ]

  return (
    <Stack>
      <Stack className={styles.header} direction="row" alignItems="center" justifyContent="space-between">
        {removingTagType || (removingTags.length > 0 && removingTags.length === tags.length && addingTags.length === 0) ? (
          <>
            <Typography className={styles.title} variant="h4">
              <del>{type.name}</del>
            </Typography>
            <Stack direction="row" alignItems="center">
              <Tooltip title="元に戻す" placement="right">
                <IconButton size="small" disabled={loading} onClick={handleClickRestoreTagType}>
                  <AddCircleOutlineIcon fontSize="inherit" />
                </IconButton>
              </Tooltip>
            </Stack>
          </>
        ) : (
          <>
            <Typography className={styles.title} variant="h4">
              {type.name}
            </Typography>
            <Stack direction="row" alignItems="center">
              <Tooltip title="削除" placement="right">
                <IconButton size="small" disabled={loading} onClick={handleClickRemoveTagType}>
                  <RemoveCircleOutlineIcon fontSize="inherit" />
                </IconButton>
              </Tooltip>
            </Stack>
          </>
        )}
      </Stack>
      <Stack spacing={0.5}>
        {allTags.map(tag => (
          <Stack key={tag.id} direction="row" alignItems="center" justifyContent="space-between">
            {removingTags.some(({ id }) => id === tag.id) ? (
              <>
                <TagBreadcrumbsList className={styles.removed} tag={tag} />
                <Tooltip title="元に戻す" placement="right">
                  <IconButton size="small" disabled={loading} onClick={() => handleClickRestoreTag(tag)}>
                    <AddCircleOutlineIcon fontSize="inherit" />
                  </IconButton>
                </Tooltip>
              </>
            ) : (
              <>
                <TagBreadcrumbsList tag={tag} />
                <Tooltip title="削除" placement="right">
                  <IconButton size="small" disabled={loading} onClick={() => handleClickRemoveTag(tag)} >
                    <RemoveCircleOutlineIcon fontSize="inherit" />
                  </IconButton>
                </Tooltip>
              </>
            )}
          </Stack>
        ))}
        <Stack flexGrow={1} direction="row">
          <AutocompleteTag
            className={styles.newTag}
            size="small"
            variant="standard"
            fullWidth
            autoHighlight
            blurOnSelect
            clearOnBlur={false}
            clearOnEscape
            includeInputInList
            focus={focus}
            selector
            forcePopupIcon={false}
            placeholder="タグの追加..."
            disabled={loading}
            renderOption={renderTagOption}
            value={null}
            getOptionDisabled={({ id }) => tags.some(tag => tag.id === id) || addingTags.some(tag => tag.id === id)}
            icon={({ ...props }) => <SellIcon {...props} />}
            onChange={handleChangeNewTag}
            slotProps={{
              popper: {
                className: styles.newTagPopper,
                placement: 'bottom-start',
              },
            }}
          />
        </Stack>
      </Stack>
    </Stack>
  )
}

export interface MediumItemMetadataTagGroupEditProps {
  loading: boolean
  type: TagType
  tags: Tag[]
  focus?: boolean
  removingTagType: boolean
  removeTagType: (type: TagType) => void
  restoreTagType: (type: TagType) => void
  addingTags: Tag[]
  removingTags: Tag[]
  addTag: (type: TagType, tag: Tag) => void
  removeTag: (type: TagType, tag: Tag) => void
  restoreTag: (type: TagType, tag: Tag) => void
}

export default MediumItemMetadataTagGroupEdit
