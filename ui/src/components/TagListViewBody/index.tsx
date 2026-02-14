'use client'

import type { FunctionComponent } from 'react'
import { useCallback, useMemo, useState } from 'react'
import clsx from 'clsx'
import { skipToken } from '@apollo/client/react'
import Grid from '@mui/material/Grid'

import TagDeleteDialog from '@/components/TagDeleteDialog'
import type { TagColumn, TagColumnSelectable } from '@/components/TagListColumn'
import TagListColumn from '@/components/TagListColumn'
import TagListColumnBodyCreate from '@/components/TagListColumnBodyCreate'
import TagListColumnBodyEdit from '@/components/TagListColumnBodyEdit'
import TagListColumnBodyList from '@/components/TagListColumnBodyList'
import { useTag } from '@/hooks'
import { Tag } from '@/types'

import styles from './styles.module.scss'

export function* ancestors(tag: Tag): Generator<Tag> {
  const tags: Tag[] = []
  for (let current: Tag | null = tag; current; current = current.parent ?? null) {
    tags.push(current)
  }

  for (let i = tags.length - 1, current = tags[i]; current; current = tags[--i]) {
    yield current
  }
}

const TagListViewBody: FunctionComponent<TagListViewBodyProps> = ({
  initial,
  readonly,
  dense,
  disabled,
  onSelect,
  selectable,
}) => {
  const tag = useTag(initial ? { id: initial.id } : skipToken)
  const initialColumns = useMemo(
    () => tag
      ? [ ...ancestors(tag) ]
          .map((tag, index, hierarchy) => ({
            index,
            creating: false,
            editing: null,
            selected: index === hierarchy.length - 1,
            parent: tag.parent ?? null,
            active: index === hierarchy.length - 1 ? null : tag,
            hit: null,
            hitInput: '',
          }))
      : [
          {
            index: 0,
            creating: false,
            editing: null,
            selected: true,
            parent: null,
            active: null,
            hit: null,
            hitInput: '',
          },
        ],
    [ tag ],
  ) satisfies TagColumn[]

  const [ columns, setColumns ] = useState<readonly TagColumn[]>(initialColumns)
  const [ creating, setCreating ] = useState(false)

  const [ selectedTag, setSelectedTag ] = useState<Tag | null>(null)
  const [ creatingParentTag, setCreatingParentTag ] = useState<Tag | null>(null)
  const [ editingTag, setEditingTag ] = useState<Tag | null>(null)
  const [ deletingTag, setDeletingTag ] = useState<Tag | null>(null)

  const setColumn = useCallback((column: TagColumn) => {
    if (!column.creating) {
      setCreating(false)
      setCreatingParentTag(null)
    }

    if (!column.editing) {
      setEditingTag(null)
    }

    setColumns(columns => {
      const currentColumn = columns[column.index]
      if (column.creating === currentColumn?.creating
        && column.editing === currentColumn.editing
        && column.selected === currentColumn.selected
        && column.parent === currentColumn.parent
        && column.active === currentColumn.active
        && column.hit?.id === currentColumn.hit?.id
        && column.hitInput === currentColumn.hitInput
        && columns.reduce((acc, c) => acc + Number(c.selected), 0) === 1
      ) {
        return columns
      }

      const newColumns = [
        ...columns.slice(0, column.index),
        column,
        ...columns.slice(column.index + 1),
      ]
      return newColumns.map(c => ({
        ...c,
        creating: column.creating ? c.index === column.index : c.creating,
        editing: column.editing ? c.index === column.index ? column.editing : null : c.editing,
        selected: column.selected ? c.index === column.index : c.selected,
      }))
    })
  }, [])

  const appendColumn = useCallback((column: TagColumn) => {
    setColumns(columns => {
      const newColumns = [ ...columns.slice(0, column.index), column ]
      const navigating = columns[column.index]?.parent?.id === column.parent?.id
      if (navigating) {
        newColumns.push(...columns.slice(column.index + 1))
      }
      return newColumns.map(c => ({
        ...c,
        active: c.active ?? (navigating ? columns[c.index]?.active ?? null : null),
        selected: c.index === column.index,
      }))
    })
  }, [])

  const closeCreateTag = useCallback(() => {
    const column = columns.find(c => c.creating)
    if (!column) {
      return
    }

    setCreating(false)
    setCreatingParentTag(null)
    setColumn({
      ...column,
      creating: false,
      editing: null,
      selected: true,
    })
  }, [ columns, setColumn ])

  const closeEditTag = useCallback(() => {
    const column = columns.find(c => c.editing)
    if (!column) {
      return
    }

    setEditingTag(null)
    setColumn({
      ...column,
      creating: false,
      editing: null,
      selected: true,
    })
  }, [ columns, setColumn ])

  const closeTag = useCallback((tag: Tag) => {
    if (creatingParentTag?.id === tag.id) {
      closeCreateTag()
    }
    if (editingTag?.id === tag.id) {
      closeEditTag()
    }

    setColumns(columns => {
      const column = columns.find(c => c.active?.id === tag.id)
      if (!column) {
        return columns
      }

      return [
        ...columns.slice(0, column.index),
        {
          ...column,
          selected: true,
          active: column.active?.id !== tag.id ? column.active : null,
          hit: column.hit?.id !== tag.id ? column.hit : null,
          hitInput: '',
        },
      ]
    })
  }, [ creatingParentTag, editingTag, closeCreateTag, closeEditTag ])

  const closeDeleteTag = useCallback(() => {
    setDeletingTag(null)
  }, [])

  const createTag = useCallback((parent: Tag | null, columnIndex: number) => {
    const column = columns[columnIndex]
    if (!column) {
      return
    }

    closeEditTag()

    setCreating(true)
    setCreatingParentTag(parent)
    setColumn({
      ...column,
      creating: true,
      editing: null,
      selected: false,
    })
  }, [ columns, closeEditTag, setColumn ])

  const editTag = useCallback((tag: Tag, columnIndex: number) => {
    const column = columns[columnIndex]
    if (!column) {
      return
    }

    closeCreateTag()

    setEditingTag(tag)
    setColumn({
      ...column,
      creating: false,
      editing: tag,
      selected: false,
    })
  }, [ columns, closeCreateTag, setColumn ])

  const deleteTag = useCallback((tag: Tag, columnIndex: number) => {
    const column = columns[columnIndex]
    if (!column) {
      return
    }

    setDeletingTag(tag)
  }, [ columns ])

  const handleCreatingTag = useCallback(() => {
    onSelect?.(null)
    setSelectedTag(null)
  }, [ onSelect ])

  const handleCreateTag = useCallback((tag: Tag) => {
    onSelect?.(tag)
    setSelectedTag(tag)
  }, [ onSelect ])

  const handleMoveTag = useCallback((tag: Tag) => {
    closeTag(tag)
  }, [ closeTag ])

  const handleDeleteTag = useCallback((tag: Tag) => {
    if (selectedTag?.id === tag.id) {
      onSelect?.(null)
      setSelectedTag(null)
    }
    if (editingTag?.id === tag.id) {
      closeEditTag()
    }

    closeTag(tag)
  }, [ closeTag, selectedTag, editingTag, onSelect, closeEditTag ])

  const handleHitTag = useCallback((hit: Tag | null) => {
    closeCreateTag()
    closeEditTag()

    if (hit) {
      setColumns([ ...ancestors(hit), null ]
        .map((tag, index, hierarchy) => ({
          index,
          creating: false,
          editing: null,
          selected: index === hierarchy.length - 1,
          parent: tag ? tag.parent ?? null : hit,
          active: index === hierarchy.length - 1 ? null : tag,
          hit: index === 0 ? hit : null,
          hitInput: index === 0 ? hit.name : '',
        })))
    } else {
      setColumns(initialColumns)
    }
  }, [ closeCreateTag, closeEditTag, setColumns, initialColumns ])

  const handleSelectTag = useCallback((tag: Tag | null) => {
    onSelect?.(tag)
    setSelectedTag(tag)

    setColumns(columns => columns.map(column => ({
      ...column,
      hit: null,
      hitInput: '',
    })))
  }, [ onSelect ])

  const showsForm = creating || Boolean(editingTag)
  const shownColumns = showsForm
    ? columns.slice(0, columns.findIndex(c => c.creating || c.editing) + 1)
    : columns

  return (
    <Grid className={styles.wrapper} container>
      {shownColumns.map(column => (
        <TagListColumn key={column.index} className={clsx(styles.column, styles.listColumn)} size={{ xs: 4, lg: 3 }} focus={column.selected}>
          <TagListColumnBodyList
            key={shownColumns[column.index - 1]?.active?.id}
            {...column}
            readonly={Boolean(readonly)}
            dense={Boolean(dense)}
            disabled={disabled}
            onHit={handleHitTag}
            onSelect={handleSelectTag}
            selectable={selectable}
            create={createTag}
            edit={editTag}
            delete={deleteTag}
            setColumn={setColumn}
            appendColumn={appendColumn}
          />
        </TagListColumn>
      ))}
      <TagListColumn key={creatingParentTag?.id ?? editingTag?.id} className={styles.column} size={{ xs: 8, lg: 9 }} focus={showsForm}>
        {creating ? (
          <TagListColumnBodyCreate parent={creatingParentTag} close={closeCreateTag} onCreating={handleCreatingTag} onCreate={handleCreateTag} />
        ) : null}
        {editingTag ? (
          <TagListColumnBodyEdit tag={editingTag} close={closeEditTag} onMove={handleMoveTag} />
        ) : null}
        {deletingTag ? (
          <TagDeleteDialog key={deletingTag.id} tag={deletingTag} close={closeDeleteTag} onDelete={handleDeleteTag} />
        ) : null}
      </TagListColumn>
    </Grid>
  )
}

export interface TagListViewBodyProps {
  readonly initial?: Tag
  readonly readonly?: boolean
  readonly dense?: boolean
  readonly disabled?: (tag: Tag) => boolean
  readonly onSelect?: (tag: Tag | null) => void
  readonly selectable?: TagColumnSelectable
}

export default TagListViewBody
