'use client'

import type { FunctionComponent } from 'react'
import { useCallback, useState } from 'react'
import clsx from 'clsx'
import Card from '@mui/material/Card'
import Grid from '@mui/material/Unstable_Grid2'

import type { TagTypeColumn } from '@/components/TagTypeListColumn'
import TagTypeDeleteDialog from '@/components/TagTypeDeleteDialog'
import TagTypeListColumn from '@/components/TagTypeListColumn'
import TagTypeListColumnBodyCreate from '@/components/TagTypeListColumnBodyCreate'
import TagTypeListColumnBodyEdit from '@/components/TagTypeListColumnBodyEdit'
import TagTypeListColumnBodyList from '@/components/TagTypeListColumnBodyList'
import TagTypeListColumnBodyShow from '@/components/TagTypeListColumnBodyShow'
import type { TagType } from '@/types'

import styles from './styles.module.scss'

const TagTypeListView: FunctionComponent<TagTypeListViewProps> = ({
  className,
  initial,
  readonly,
  dense,
  disabled,
  onSelect,
}) => {
  const [ column, setColumn ] = useState<TagTypeColumn>({
    creating: false,
    editing: null,
    active: initial ?? null,
  })

  const [ creating, setCreating ] = useState(false)
  const [ showingTagType, setShowingTagType ] = useState<TagType | null>(initial ?? null)
  const [ editingTagType, setEditingTagType ] = useState<TagType | null>(null)
  const [ deletingTagType, setDeletingTagType ] = useState<TagType | null>(null)

  const createTagType = useCallback(() => {
    setCreating(true)
    setShowingTagType(null)
    setEditingTagType(null)
    setColumn(column => ({
      ...column,
      creating: true,
      editing: null,
    }))
  }, [])

  const closeCreateTagType = useCallback(() => {
    setCreating(false)
    setShowingTagType(column.active)
    setEditingTagType(null)
    setColumn(column => ({
      ...column,
      creating: false,
      editing: null,
    }))
  }, [ column ])

  const showTagType = useCallback((tagType: TagType) => {
    setCreating(false)
    setShowingTagType(tagType)
    setEditingTagType(null)
    setColumn(column => ({
      ...column,
      creating: false,
      editing: null,
      active: tagType,
    }))
  }, [])

  const closeShowTagType = useCallback(() => {
    setCreating(false)
    setShowingTagType(null)
    setEditingTagType(null)
    setColumn(column => ({
      ...column,
      creating: false,
      editing: null,
      active: null,
    }))
  }, [])

  const editTagType = useCallback((tagType: TagType) => {
    setCreating(false)
    setShowingTagType(null)
    setEditingTagType(tagType)
    setColumn(column => ({
      ...column,
      creating: false,
      editing: tagType,
    }))
  }, [])

  const closeEditTagType = useCallback(() => {
    setCreating(false)
    setShowingTagType(column.active)
    setEditingTagType(null)
    setColumn(column => ({
      ...column,
      creating: false,
      editing: null,
    }))
  }, [ column ])

  const handleEditTagType = useCallback((tagType: TagType) => {
    if (column.active?.id === tagType.id) {
      setShowingTagType(tagType)
    }
  }, [ column ])

  const deleteTagType = useCallback((tagType: TagType) => {
    setDeletingTagType(tagType)
  }, [])

  const closeDeleteTagType = useCallback(() => {
    setDeletingTagType(null)
  }, [])

  const handleDeleteTagType = useCallback((tagType: TagType) => {
    if (showingTagType?.id == tagType.id) {
      closeShowTagType()
    }
    if (editingTagType?.id == tagType.id) {
      closeEditTagType()
    }
  }, [ showingTagType, closeShowTagType, editingTagType, closeEditTagType ])

  return (
    <Card className={clsx(styles.container, className)}>
      <Grid className={styles.wrapper} container>
        <TagTypeListColumn className={clsx(styles.column, styles.listColumn)} xs={4} lg={3}>
          <TagTypeListColumnBodyList
            {...column}
            readonly={Boolean(readonly)}
            dense={Boolean(dense)}
            disabled={disabled}
            onSelect={onSelect}
            create={createTagType}
            show={showTagType}
            edit={editTagType}
            delete={deleteTagType}
          />
        </TagTypeListColumn>
        <TagTypeListColumn key={showingTagType?.id ?? editingTagType?.id ?? String(creating)} className={styles.column} xs={8} lg={9}>
          {showingTagType ? (
            <TagTypeListColumnBodyShow tagType={showingTagType} edit={editTagType} />
          ) : null}
          {creating ? (
            <TagTypeListColumnBodyCreate close={closeCreateTagType} />
          ) : null}
          {editingTagType ? (
            <TagTypeListColumnBodyEdit tagType={editingTagType} close={closeEditTagType} onEdit={handleEditTagType} />
          ) : null}
        </TagTypeListColumn>
      </Grid>
      {deletingTagType ? (
        <TagTypeDeleteDialog key={deletingTagType.id} tagType={deletingTagType} close={closeDeleteTagType} onDelete={handleDeleteTagType} />
      ) : null}
    </Card>
  )
}

export interface TagTypeListViewProps {
  className?: string
  initial?: TagType,
  readonly?: boolean
  dense?: boolean
  disabled?: (tagType: TagType) => boolean
  onSelect?: (tagType: TagType) => void
}

export default TagTypeListView
