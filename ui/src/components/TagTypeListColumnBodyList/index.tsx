'use client'

import type { ComponentPropsWithoutRef, FunctionComponent, MouseEvent, SyntheticEvent } from 'react'
import { useCallback, useState } from 'react'
import clsx from 'clsx'
import type { AutocompleteInputChangeReason } from '@mui/material/Autocomplete'
import IconButton from '@mui/material/IconButton'
import List from '@mui/material/List'
import Stack from '@mui/material/Stack'
import AddIcon from '@mui/icons-material/Add'
import DeleteOutlinedIcon from '@mui/icons-material/DeleteOutlined'
import EditOutlinedIcon from '@mui/icons-material/EditOutlined'
import LabelIcon from '@mui/icons-material/Label'
import SearchIcon from '@mui/icons-material/Search'

import AutocompleteTagType from '@/components/AutocompleteTagType'
import TagTypeListColumnBodyListItem from '@/components/TagTypeListColumnBodyListItem'
import { useAllTagTypes } from '@/hooks'
import type { TagType } from '@/types'

import styles from './styles.module.scss'

const TagTypeListColumnBodyList: FunctionComponent<TagTypeListColumnBodyListProps> = ({
  creating,
  editing,
  active,
  hit,
  hitInput,
  readonly,
  dense,
  disabled: disabledTagType,
  onSelect: onSelectTagType,
  onHit: onHitTagType,
  show: showTagType,
  create: createTagType,
  edit: editTagType,
  delete: deleteTagType,
  setColumn,
}) => {
  const allTagTypes = useAllTagTypes()

  const [ scrollTop, setScrollTop ] = useState(0)
  const ref = useCallback((node: HTMLElement | null) => {
    if (!node) {
      return
    }
    if (creating) {
      setScrollTop(node.scrollTop)
      node.scrollTo({
        top: node.scrollHeight,
        behavior: 'smooth',
      })
    } else {
      node.scrollTo({
        top: scrollTop,
        behavior: 'smooth',
      })
    }
  }, [ creating, scrollTop ])

  const handleClickTagType = (tagType: TagType) => {
    onSelectTagType?.(tagType)
    onHitTagType?.(null)
    showTagType(tagType)
  }

  const handleHitTagType = useCallback((tagType: TagType | null) => {
    onHitTagType?.(tagType)
  }, [ onHitTagType ])

  const handleInputHitTagType = useCallback((_e: SyntheticEvent, value: string, reason: AutocompleteInputChangeReason) => {
    if (!value && reason === 'input') {
      onHitTagType?.(null)
    }
    setColumn({
      creating,
      editing,
      active,
      hit,
      hitInput: value,
    })
  }, [ onHitTagType, setColumn, creating, editing, active, hit ])

  const handleClickCreateTagType = useCallback(() => {
    createTagType()
  }, [ createTagType ])

  const handleClickEditTagType = (e: MouseEvent<HTMLButtonElement>, tagType: TagType) => {
    editTagType(tagType)
    e.stopPropagation()
  }

  const handleClickDeleteTagType = (e: MouseEvent<HTMLButtonElement>, tagType: TagType) => {
    deleteTagType(tagType)
    e.stopPropagation()
  }

  const handleMouseDownEditTagType = useCallback((e: MouseEvent<HTMLButtonElement>) => {
    e.stopPropagation()
  }, [])

  const handleMouseDownDeleteTagType = useCallback((e: MouseEvent<HTMLButtonElement>) => {
    e.stopPropagation()
  }, [])

  const renderTagTypeOption = useCallback(({ key, ...props }: ComponentPropsWithoutRef<'li'>, option: TagType) => (
    <li key={key} {...props}>
      <Stack direction="row" spacing={0.5} alignItems="start">
        <LabelIcon className={styles.tagTypeSearchIcon} fontSize="small" />
        <span className={styles.tagTypeSearchText}>{option.name}</span>
      </Stack>
    </li>
  ), [])

  if (!allTagTypes) {
    throw new Error('unreachable')
  }

  return (
    <Stack className={styles.container}>
      <Stack className={clsx(styles.title, !readonly && styles.buttons)}>
        <Stack direction="row" spacing={1} alignItems="center" justifyContent="space-between">
          <AutocompleteTagType
            className={styles.tagTypeSearch}
            size="small"
            variant="standard"
            fullWidth
            autoHighlight
            blurOnSelect
            clearOnBlur={false}
            clearOnEscape
            includeInputInList
            forcePopupIcon={false}
            placeholder="検索"
            renderOption={renderTagTypeOption}
            value={hit}
            inputValue={hitInput}
            icon={({ ...props }) => <SearchIcon fontSize="small" {...props} />}
            onChange={handleHitTagType}
            onInputChange={handleInputHitTagType}
            slotProps={{
              popper: {
                className: styles.tagTypeSearchPopper,
                placement: 'bottom-start',
              },
            }}
          />
          {!readonly ? (
            <IconButton size="small" onClick={handleClickCreateTagType}>
              <AddIcon />
            </IconButton>
          ) : null}
        </Stack>
      </Stack>
      <List ref={ref} dense={dense} className={styles.tagTypes}>
        {allTagTypes.map(tagType => (
          <TagTypeListColumnBodyListItem
            key={tagType.id}
            className={styles.tagType}
            dense={dense}
            disabled={Boolean(disabledTagType?.(tagType))}
            selected={!creating && (editing ?? active)?.id === tagType.id}
            primary={tagType.name}
            secondary={dense ? null : tagType.kana || null}
            onClick={() => handleClickTagType(tagType)}
          >
            {!readonly ? (
              <>
                <IconButton
                  className={styles.tagTypeButton}
                  size="small"
                  onMouseDown={handleMouseDownEditTagType}
                  onClick={e => handleClickEditTagType(e, tagType)}
                >
                  <EditOutlinedIcon fontSize={dense ? 'small' : 'medium'} />
                </IconButton>
                <IconButton
                  className={styles.tagTypeButton}
                  size="small"
                  onMouseDown={handleMouseDownDeleteTagType}
                  onClick={e => handleClickDeleteTagType(e, tagType)}
                >
                  <DeleteOutlinedIcon fontSize={dense ? 'small' : 'medium'} />
                </IconButton>
              </>
            ) : null}
          </TagTypeListColumnBodyListItem>
        ))}
        {creating ? (
          <TagTypeListColumnBodyListItem
            className={styles.tagType}
            dense={dense}
            selected
            primary="新しいタイプ"
          />
        ) : null}
      </List>
    </Stack>
  )
}

export interface TagTypeColumn {
  creating: boolean
  editing: TagType | null
  active: TagType | null
  hit: TagType | null
  hitInput: string
}

export interface TagTypeListColumnBodyListProps extends TagTypeColumn {
  readonly: boolean
  dense: boolean
  disabled?: (tagType: TagType) => boolean
  onHit?: (tagType: TagType | null) => void
  onSelect?: (tagType: TagType) => void
  create: () => void
  show: (tagType: TagType) => void
  edit: (tagType: TagType) => void
  delete: (tagType: TagType) => void
  setColumn: (column: TagTypeColumn) => void
}

export default TagTypeListColumnBodyList
