'use client'

import type { ComponentPropsWithoutRef, FunctionComponent, SyntheticEvent } from 'react'
import { useCallback, useState } from 'react'
import Button from '@mui/material/Button'
import Stack from '@mui/material/Stack'
import LabelIcon from '@mui/icons-material/Label'

import AutocompleteTagType from '@/components/AutocompleteTagType'
import MediumItemMetadataHeader from '@/components/MediumItemMetadataHeader'
import MediumItemMetadataTagGroupEdit from '@/components/MediumItemMetadataTagGroupEdit'
import type { TagTagTypeInput } from '@/graphql/types.generated'
import { useBeforeUnload } from '@/hooks'
import type { Medium, Tag, TagType } from '@/types'

import styles from './styles.module.scss'

const hasChanges = (addingTagTypes: TagType[], removingTagTypes: TagType[], addingTags: Map<TagTypeID, Tag[]>, removingTags: Map<TagTypeID, Tag[]>) => {
  if (addingTagTypes.length > 0 || removingTagTypes.length > 0) {
    return true
  }

  for (const tags of addingTags.values()) {
    if (tags.length > 0) {
      return true
    }
  }

  for (const tags of removingTags.values()) {
    if (tags.length > 0) {
      return true
    }
  }

  return false
}

const MediumItemMetadataTagEdit: FunctionComponent<MediumItemMetadataTagEditProps> = ({
  loading,
  focus,
  medium,
  save,
  close,
}) => {
  const [ focusedTagType, setFocusedTagType ] = useState<TagType | null>(null)
  const [ newTagTypeInput, setNewTagTypeInput ] = useState('')

  const [ addingTagTypes, setAddingTagTypes ] = useState<TagType[]>([])
  const [ removingTagTypes, setRemovingTagTypes ] = useState<TagType[]>([])

  const [ addingTags, setAddingTags ] = useState(new Map<TagTypeID, Tag[]>())
  const [ removingTags, setRemovingTags ] = useState(new Map<TagTypeID, Tag[]>())

  const tags = medium.tags ?? []
  const groups = tags.reduce<TagGroup[]>((groups, { tag, type }) => {
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

  const handleChangeNewTagType = useCallback((type: TagType | null) => {
    if (!type) {
      return
    }

    setNewTagTypeInput('')

    setFocusedTagType(type)
    setAddingTagTypes(addingTagTypes => [
      ...addingTagTypes,
      type,
    ])
  }, [])

  const handleChangeNewTagTypeInput = useCallback((_e: SyntheticEvent, value: string) => {
    setNewTagTypeInput(value)
  }, [])

  const removeTagType = useCallback((type: TagType) => {
    setFocusedTagType(null)

    setAddingTagTypes(addingTagTypes => {
      const idx = addingTagTypes.findIndex(({ id }) => id === type.id)
      if (idx < 0) {
        return addingTagTypes
      }

      return addingTagTypes.toSpliced(idx, 1)
    })

    if (!groups.some(group => group.type.id === type.id)) {
      return
    }

    setRemovingTagTypes(removingTagTypes => [
      ...removingTagTypes,
      type,
    ])
  }, [ groups ])

  const restoreTagType = useCallback((type: TagType) => {
    setRemovingTagTypes(removingTagTypes => removingTagTypes.filter(({ id }) => id !== type.id))
  }, [])

  const addTag = useCallback((type: TagType, tag: Tag) => {
    setFocusedTagType(type)

    setAddingTags(addingTags => {
      const newTags = addingTags.get(type.id) ?? []
      if (newTags.some(({ id }) => id === tag.id)) {
        return addingTags
      }
      if (groups.some(group => group.type.id === type.id && group.tags.some(({ id }) => id === tag.id))) {
        return addingTags
      }

      return new Map(addingTags).set(type.id, [ ...newTags, tag ])
    })

    setRemovingTags(removingTags => {
      const newTags = removingTags.get(type.id) ?? []
      const idx = newTags.findIndex(({ id }) => id === tag.id)
      if (idx < 0) {
        return removingTags
      }

      return new Map(removingTags).set(type.id, newTags.toSpliced(idx, 1))
    })
  }, [ groups ])

  const removeTag = useCallback((type: TagType, tag: Tag) => {
    setFocusedTagType(null)

    setAddingTags(addingTags => {
      const newTags = addingTags.get(type.id) ?? []
      const idx = newTags.findIndex(({ id }) => id === tag.id)
      if (idx < 0) {
        return addingTags
      }

      return new Map(addingTags).set(type.id, newTags.toSpliced(idx, 1))
    })

    if (!groups.some(group => group.type.id === type.id && group.tags.some(({ id }) => id === tag.id))) {
      return
    }

    setRemovingTags(removingTags => {
      const newTags = removingTags.get(type.id) ?? []
      if (newTags.some(({ id }) => id === tag.id)) {
        return removingTags
      }

      return new Map(removingTags).set(type.id, [ ...newTags, tag ])
    })
  }, [ groups ])

  const restoreTag = useCallback((type: TagType, tag: Tag) => {
    setFocusedTagType(null)

    setRemovingTags(removingTags => {
      const newTags = removingTags.get(type.id) ?? []
      const idx = newTags.findIndex(({ id }) => id === tag.id)
      if (idx < 0) {
        return removingTags
      }

      return new Map(removingTags).set(type.id, newTags.toSpliced(idx, 1))
    })
  }, [])

  const handleClickCancel = useCallback(() => {
    close?.()
  }, [ close ])

  const handleClickSubmit = useCallback(async () => {
    const addTagTagTypeIDs: TagTagTypeInput[] = []
    for (const [ tagTypeId, tags ] of addingTags) {
      addTagTagTypeIDs.push(...tags.map(({ id: tagId }) => ({ tagTypeId, tagId })))
    }

    const removeTagTagTypeIDs: TagTagTypeInput[] = []
    for (const [ tagTypeId, tags ] of removingTags) {
      removeTagTagTypeIDs.push(...tags.map(({ id: tagId }) => ({ tagTypeId, tagId })))
    }

    try {
      await save(medium.id, addTagTagTypeIDs, removeTagTagTypeIDs)
      close?.()
    } catch (e) {
      console.error('Error updating medium\n', e)
    }
  }, [ save, medium, addingTags, removingTags, close ])

  const renderTagTypeOption = useCallback(({ key, ...props }: ComponentPropsWithoutRef<'li'>, option: TagType) => (
    <li key={key} {...props}>
      <Stack direction="row" spacing={0.5} alignItems="start">
        <LabelIcon className={styles.tagTypeSearchIcon} fontSize="small" />
        <span className={styles.tagTypeSearchText}>{option.name}</span>
      </Stack>
    </li>
  ), [])

  const changed = hasChanges(addingTagTypes, removingTagTypes, addingTags, removingTags)
  useBeforeUnload(changed)

  return (
    <Stack>
      <MediumItemMetadataHeader title="タグ">
        <Button onClick={handleClickSubmit} loading={loading}>
          保存
        </Button>
        <Button onClick={handleClickCancel}>
          キャンセル
        </Button>
      </MediumItemMetadataHeader>
      <Stack spacing={4}>
        {groups.map(({ type, tags }) => (
          <MediumItemMetadataTagGroupEdit
            key={`${type.id}-${String(addingTags.get(type.id)?.length ?? 0)}`}
            loading={loading}
            type={type}
            tags={tags}
            focus={focusedTagType?.id === type.id}
            removingTagType={removingTagTypes.some(({ id }) => id === type.id)}
            removeTagType={removeTagType}
            restoreTagType={restoreTagType}
            addingTags={addingTags.get(type.id) ?? []}
            removingTags={removingTags.get(type.id) ?? []}
            addTag={addTag}
            removeTag={removeTag}
            restoreTag={restoreTag}
          />
        ))}
        {addingTagTypes.map(type => (
          <MediumItemMetadataTagGroupEdit
            key={`${type.id}-${String(addingTags.get(type.id)?.length ?? 0)}`}
            loading={loading}
            type={type}
            tags={[]}
            focus={focusedTagType?.id === type.id}
            removingTagType={false}
            removeTagType={removeTagType}
            restoreTagType={restoreTagType}
            addingTags={addingTags.get(type.id) ?? []}
            removingTags={removingTags.get(type.id) ?? []}
            addTag={addTag}
            removeTag={removeTag}
            restoreTag={restoreTag}
          />
        ))}
        <Stack spacing={0.5} direction="row" alignItems="center" justifyContent="space-between">
          <AutocompleteTagType
            fullWidth
            openOnFocus
            autoHighlight
            blurOnSelect
            clearOnBlur={false}
            clearOnEscape
            includeInputInList
            focus={focus && groups.length === 0}
            loadOnOpen
            placeholder="タイプの追加..."
            disabled={loading}
            renderOption={renderTagTypeOption}
            value={null}
            inputValue={newTagTypeInput}
            getOptionDisabled={({ id }) => groups.some(group => group.type.id === id) || addingTagTypes.some(type => type.id === id)}
            icon={({ ...props }) => <LabelIcon {...props} />}
            onChange={handleChangeNewTagType}
            onInputChange={handleChangeNewTagTypeInput}
          />
        </Stack>
      </Stack>
    </Stack>
  )
}

export interface MediumItemMetadataTagEditProps {
  loading: boolean
  focus?: boolean
  medium: Medium
  save: (id: string, addTagTagTypeIDs: TagTagTypeInput[], removeTagTagTypeIDs: TagTagTypeInput[]) => Promise<Medium>
  close?: () => void
}

interface TagGroup {
  type: TagType
  tags: Tag[]
}

type TagTypeID = string

export default MediumItemMetadataTagEdit
