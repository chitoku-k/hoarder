'use client'

import type { ComponentPropsWithoutRef, FunctionComponent, MouseEvent, SyntheticEvent } from 'react'
import { useCallback, useState, useTransition } from 'react'
import { useListFormatter } from '@react-aria/i18n'
import clsx from 'clsx'
import type { AutocompleteInputChangeReason } from '@mui/material/Autocomplete'
import Button from '@mui/material/Button'
import IconButton from '@mui/material/IconButton'
import List from '@mui/material/List'
import Stack from '@mui/material/Stack'
import AddIcon from '@mui/icons-material/Add'
import DeleteOutlinedIcon from '@mui/icons-material/DeleteOutlined'
import EditOutlinedIcon from '@mui/icons-material/EditOutlined'
import ExpandMoreIcon from '@mui/icons-material/ExpandMore'
import SearchIcon from '@mui/icons-material/Search'

import AutocompleteTag from '@/components/AutocompleteTag'
import TagBreadcrumbsList from '@/components/TagBreadcrumbsList'
import TagListColumnBodyListChildren from '@/components/TagListColumnBodyListChildren'
import TagListColumnBodyListItem from '@/components/TagListColumnBodyListItem'
import TagListColumnBodyListRoot from '@/components/TagListColumnBodyListRoot'
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

  const handleClickTag = useCallback((tag: Tag) => {
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
  }, [ appendColumn, onSelectTag, setColumn, index, parent ])

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
  }, [ onHitTag, setColumn, index, creating, editing, selected, parent, active ])

  const handleClickSelectTag = useCallback(() => {
    onSelectTag?.(parent)
  }, [ onSelectTag, parent ])

  const handleClickMore = useCallback((fetchMore: () => Promise<void>) => {
    startTransition(async () => {
      await fetchMore()
    })
  }, [])

  const handleClickCreateTag = useCallback(() => {
    createTag(parent, index)
  }, [ createTag, parent, index ])

  const handleClickEditTag = useCallback((e: MouseEvent<HTMLButtonElement>, tag: Tag) => {
    editTag(tag, index)
    e.stopPropagation()
  }, [ editTag, index ])

  const handleClickDeleteTag = useCallback((e: MouseEvent<HTMLButtonElement>, tag: Tag) => {
    deleteTag(tag, index)
    e.stopPropagation()
  }, [ deleteTag, index ])

  const handleMouseDownEditTag = useCallback((e: MouseEvent<HTMLButtonElement>) => {
    e.stopPropagation()
  }, [])

  const handleMouseDownDeleteTag = useCallback((e: MouseEvent<HTMLButtonElement>) => {
    e.stopPropagation()
  }, [])

  const tagSecondaryNode = useCallback((kana: string, aliases: readonly string[]) => {
    if (!kana && !aliases.length) {
      return null
    }
    if (!aliases.length) {
      return kana
    }
    return (
      <>
        {kana}<br />{formatter.format(aliases)}
      </>
    )
  }, [ formatter ])

  const renderTagItem = useCallback((tag: Tag) => (
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
  ), [
    readonly,
    dense,
    creating,
    editing,
    active,
    disabledTag,
    tagSecondaryNode,
    handleClickTag,
    handleMouseDownEditTag,
    handleClickEditTag,
    handleMouseDownDeleteTag,
    handleClickDeleteTag,
  ])

  const renderTagItems = useCallback((tags: readonly Tag[], hasNextPage?: boolean, fetchMore?: () => Promise<void>) => (
    <List ref={ref} className={styles.tags} dense={dense}>
      {tags.map(tag => renderTagItem(tag))}
      {active && tags.every(({ id }) => id !== active.id) ? renderTagItem(active) : null}
      {creating ? (
        <TagListColumnBodyListItem
          className={styles.tag}
          dense={dense}
          selected
          primary="新しいタグ"
        />
      ) : null}
      {hasNextPage && fetchMore ? (
        <Stack className={styles.tagMoreContainer}>
          <Button
            className={styles.tagMoreButton}
            color="inherit"
            loading={loading}
            endIcon={<ExpandMoreIcon />}
            onClick={() => handleClickMore(fetchMore)}
          >
            次へ
          </Button>
        </Stack>
      ) : null}
    </List>
  ), [ dense, creating, active, loading, ref, handleClickMore, renderTagItem ])

  const renderTagOption = useCallback(({ key, ...props }: ComponentPropsWithoutRef<'li'>, option: Tag) => (
    <li key={key} {...props}>
      <TagBreadcrumbsList tag={option} />
    </li>
  ), [])

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
      {parent ? (
        <TagListColumnBodyListChildren id={parent.id} component={renderTagItems} />
      ) : (
        <TagListColumnBodyListRoot number={50} component={renderTagItems} />
      )}
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
  readonly index: number
  readonly creating: boolean
  readonly editing: Tag | null
  readonly selected: boolean
  readonly parent: Tag | null
  readonly active: Tag | null
  readonly hit: Tag | null
  readonly hitInput: string
}

export type TagColumnSelectable = 'column' | 'tag'

export interface TagListColumnBodyListProps extends TagColumn {
  readonly readonly: boolean
  readonly dense: boolean
  readonly selectable?: TagColumnSelectable
  readonly disabled?: (tag: Tag) => boolean
  readonly onHit?: (tag: Tag | null) => void
  readonly onSelect?: (tag: Tag | null) => void
  readonly create: (parent: Tag | null, columnIndex: number) => void
  readonly edit: (tag: Tag, columnIndex: number) => void
  readonly delete: (tag: Tag, columnIndex: number) => void
  readonly setColumn: (column: TagColumn) => void
  readonly appendColumn: (column: TagColumn) => void
}

export default TagListColumnBodyList
