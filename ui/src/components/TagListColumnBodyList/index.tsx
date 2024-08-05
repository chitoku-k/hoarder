'use client'

import type { ComponentPropsWithoutRef, FunctionComponent, MouseEvent, SyntheticEvent } from 'react'
import { useCallback, useState, useTransition } from 'react'
import { useListFormatter } from '@react-aria/i18n'
import clsx from 'clsx'
import type { AutocompleteInputChangeReason } from '@mui/material/Autocomplete'
import Button from '@mui/material/Button'
import IconButton from '@mui/material/IconButton'
import List from '@mui/material/List'
import LoadingButton from '@mui/lab/LoadingButton'
import Stack from '@mui/material/Stack'
import AddIcon from '@mui/icons-material/Add'
import DeleteOutlinedIcon from '@mui/icons-material/DeleteOutlined'
import EditOutlinedIcon from '@mui/icons-material/EditOutlined'
import ExpandMoreIcon from '@mui/icons-material/ExpandMore'
import SearchIcon from '@mui/icons-material/Search'

import AutocompleteTag from '@/components/AutocompleteTag'
import TagBreadcrumbsList from '@/components/TagBreadcrumbsList'
import TagListColumnBodyListItem from '@/components/TagListColumnBodyListItem'
import { useTags } from '@/hooks'
import type { Tag } from '@/types'

import styles from './styles.module.scss'

const TagListColumnBodyList: FunctionComponent<TagListColumnBodyListProps> = ({
  index,
  creating,
  editing,
  selected,
  parent,
  active,
  hit,
  hitInput,
  readonly,
  dense,
  selectable,
  disabled: disabledTag,
  onHit: onHitTag,
  onSelect: onSelectTag,
  create: createTag,
  edit: editTag,
  delete: deleteTag,
  setColumn,
  appendColumn,
}) => {
  const [ loading, startTransition ] = useTransition()
  const formatter = useListFormatter({
    style: 'long',
    type: 'conjunction',
  })

  const [ children, hasNextPage, fetchMore ] = parent
    ? useTags(parent.id)
    : useTags(50)

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

  const handleClickTag = (tag: Tag) => {
    onSelectTag?.(tag)
    setColumn({
      index,
      creating: false,
      editing: null,
      selected: true,
      parent,
      active: tag,
      hit: null,
      hitInput: '',
    })
    appendColumn({
      index: index + 1,
      creating: false,
      editing: null,
      selected: true,
      parent: tag,
      active: null,
      hit: null,
      hitInput: '',
    })
  }

  const handleHitTag = useCallback((tag: Tag | null) => {
    onHitTag?.(tag)
  }, [ onHitTag ])

  const handleInputHitTag = useCallback((_e: SyntheticEvent, value: string, reason: AutocompleteInputChangeReason) => {
    if (!value && reason === 'input') {
      onHitTag?.(null)
    }
    setColumn({
      index,
      creating,
      editing,
      selected,
      parent,
      active,
      hit: null,
      hitInput: value,
    })
  }, [ onHitTag, index, creating, editing, selected, parent, active, hit ])

  const handleClickSelectTag = useCallback(() => {
    onSelectTag?.(parent)
  }, [ onSelectTag, parent ])

  const handleClickMore = useCallback(() => {
    if (!fetchMore) {
      throw new Error('No handler found to fetch more')
    }
    startTransition(() => {
      fetchMore()
    })
  }, [ fetchMore ])

  const handleClickCreateTag = useCallback(() => {
    createTag(parent, index)
  }, [ createTag, parent, index ])

  const handleClickEditTag = (e: MouseEvent<HTMLButtonElement>, tag: Tag) => {
    editTag(tag, index)
    e.stopPropagation()
  }

  const handleClickDeleteTag = (e: MouseEvent<HTMLButtonElement>, tag: Tag) => {
    deleteTag(tag, index)
    e.stopPropagation()
  }

  const handleMouseDownEditTag = useCallback((e: MouseEvent<HTMLButtonElement>) => {
    e.stopPropagation()
  }, [])

  const handleMouseDownDeleteTag = useCallback((e: MouseEvent<HTMLButtonElement>) => {
    e.stopPropagation()
  }, [])

  const tagSecondaryNode = useCallback((kana: string, aliases: string[]) => {
    if (!kana && !aliases?.length) {
      return null
    }
    if (!aliases?.length) {
      return kana
    }
    return (
      <>
        {kana}<br />{formatter.format(aliases)}
      </>
    )
  }, [ formatter ])

  const renderTagOption = useCallback(({ key, ...props }: ComponentPropsWithoutRef<'li'>, option: Tag) => (
    <li key={key} {...props}>
      <TagBreadcrumbsList tag={option} />
    </li>
  ), [])

  const tags = !active || children.some(({ id }) => id === active.id)
    ? children
    : [ ...children, active ]

  return (
    <Stack className={styles.container}>
      <Stack className={clsx(styles.title, !readonly && styles.buttons)}>
        <Stack direction="row" spacing={1} alignItems="center" justifyContent="space-between">
          {parent && !hit ? (
            <Stack className={styles.name}>{parent.name}</Stack>
          ) : (
            <AutocompleteTag
              className={styles.tagSearch}
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
              disabled={loading}
              renderOption={renderTagOption}
              value={hit}
              inputValue={hitInput}
              icon={({ ...props }) => <SearchIcon fontSize="small" {...props} />}
              onChange={handleHitTag}
              onInputChange={handleInputHitTag}
              slotProps={{
                popper: {
                  className: styles.tagSearchPopper,
                  placement: 'bottom-start',
                },
              }}
            />
          )}
          {!readonly ? (
            <IconButton size="small" onClick={handleClickCreateTag}>
              <AddIcon />
            </IconButton>
          ) : null}
        </Stack>
      </Stack>
      <List ref={ref} dense={dense} className={styles.tags}>
        {tags.map(tag => (
          <TagListColumnBodyListItem
            key={tag.id}
            className={styles.tag}
            dense={dense}
            disabled={Boolean(disabledTag?.(tag))}
            selected={!creating && (editing ?? active)?.id === tag.id}
            primary={tag.name}
            secondary={dense ? null : tagSecondaryNode(tag.kana, tag.aliases)}
            onClick={() => handleClickTag(tag)}
          >
            {!readonly ? (
              <>
                <IconButton
                  className={styles.tagButton}
                  size="small"
                  onMouseDown={handleMouseDownEditTag}
                  onClick={e => handleClickEditTag(e, tag)}
                >
                  <EditOutlinedIcon fontSize={dense ? 'small' : 'medium'} />
                </IconButton>
                <IconButton
                  className={styles.tagButton}
                  size="small"
                  onMouseDown={handleMouseDownDeleteTag}
                  onClick={e => handleClickDeleteTag(e, tag)}
                >
                  <DeleteOutlinedIcon fontSize={dense ? 'small' : 'medium'} />
                </IconButton>
              </>
            ) : null}
          </TagListColumnBodyListItem>
        ))}
        {creating ? (
          <TagListColumnBodyListItem
            className={styles.tag}
            dense={dense}
            selected
            primary="新しいタグ"
          />
        ) : null}
        {hasNextPage ? (
          <Stack className={styles.tagMoreContainer}>
            <LoadingButton
              className={styles.tagMoreButton}
              color="inherit"
              loading={loading}
              endIcon={<ExpandMoreIcon />}
              onClick={handleClickMore}
            >
              次へ
            </LoadingButton>
          </Stack>
        ) : null}
      </List>
      {selectable === 'column' ? (
        <Stack className={styles.selectButtonContainer}>
          <Button onClick={handleClickSelectTag}>
            選択
          </Button>
        </Stack>
      ) : null}
    </Stack>
  )
}

export interface TagColumn {
  index: number
  creating: boolean
  editing: Tag | null
  selected: boolean
  parent: Tag | null
  active: Tag | null
  hit: Tag | null
  hitInput: string
}

export type TagColumnSelectable = 'column' | 'tag'

export interface TagListColumnBodyListProps extends TagColumn {
  readonly: boolean
  dense: boolean
  selectable?: TagColumnSelectable
  disabled?: (tag: Tag) => boolean
  onHit?: (tag: Tag | null) => void
  onSelect?: (tag: Tag | null) => void
  create: (parent: Tag | null, columnIndex: number) => void
  edit: (tag: Tag, columnIndex: number) => void
  delete: (tag: Tag, columnIndex: number) => void
  setColumn: (column: TagColumn) => void
  appendColumn: (column: TagColumn) => void
}

export default TagListColumnBodyList
