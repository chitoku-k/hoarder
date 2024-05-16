'use client'

import type { FunctionComponent } from 'react'
import { useCallback, useState } from 'react'
import Button from '@mui/material/Button'
import LoadingButton from '@mui/lab/LoadingButton'
import Stack from '@mui/material/Stack'
import LabelIcon from '@mui/icons-material/Label'

import AutocompleteTagType from '@/components/AutocompleteTagType'
import MediumItemMetadataHeader from '@/components/MediumItemMetadataHeader'
import MediumItemMetadataTagGroupEdit from '@/components/MediumItemMetadataTagGroupEdit'
import type { Medium, Tag, TagType } from '@/types'
import { TagTagTypeInput } from '@/hooks/types.generated'

const MediumItemMetadataTagEdit: FunctionComponent<MediumItemMetadataTagEditProps> = ({
  loading,
  focus,
  medium,
  save,
  close,
}) => {
  const [ focusedTagType, setFocusedTagType ] = useState<TagType | null>(null)
  const [ newTagType, setNewTagType ] = useState<TagType | null>(null)

  const [ addingTagTypes, setAddingTagTypes ] = useState<TagType[]>([])
  const [ removingTagTypes, setRemovingTagTypes ] = useState<TagType[]>([])

  const [ addingTags, setAddingTags ] = useState(new Map<TagTypeID, Tag[]>())
  const [ removingTags, setRemovingTags ] = useState(new Map<TagTypeID, Tag[]>())

  const tags = medium.tags ?? []
  const groups = tags.reduce((groups, { tag, type }) => {
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
  }, [] as TagGroup[])

  const handleChangeNewTagType = useCallback((type: TagType | null) => {
    if (!type) {
      return
    }

    setNewTagType(null)
    setFocusedTagType(type)
    setAddingTagTypes(addingTagTypes => [
      ...addingTagTypes,
      type,
    ])
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

      newTags.push(tag)
      return new Map(addingTags.set(type.id, newTags))
    })

    setRemovingTags(removingTags => {
      const newTags = removingTags.get(type.id) ?? []
      const idx = newTags.findIndex(({ id }) => id === tag.id)
      if (idx < 0) {
        return removingTags
      }

      return new Map(removingTags.set(type.id, newTags.toSpliced(idx, 1)))
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

      return new Map(addingTags.set(type.id, newTags.toSpliced(idx, 1)))
    })

    if (!groups.some(group => group.type.id === type.id && group.tags.some(({ id }) => id === tag.id))) {
      return
    }

    setRemovingTags(removingTags => {
      const newTags = removingTags.get(type.id) ?? []
      if (newTags.some(({ id }) => id === tag.id)) {
        return removingTags
      }

      newTags.push(tag)
      return new Map(removingTags.set(type.id, newTags))
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

      return new Map(removingTags.set(type.id, newTags.toSpliced(idx, 1)))
    })
  }, [])

  const handleClickCancel = useCallback(() => {
    close?.()
  }, [ close ])

  const handleClickSubmit = useCallback(() => {
    const addTagTagTypeIDs: TagTagTypeInput[] = []
    for (const [ tagTypeId, tags ] of addingTags) {
      addTagTagTypeIDs.push(...tags.map(({ id: tagId }) => ({ tagTypeId, tagId })))
    }

    const removeTagTagTypeIDs: TagTagTypeInput[] = []
    for (const [ tagTypeId, tags ] of removingTags) {
      removeTagTagTypeIDs.push(...tags.map(({ id: tagId }) => ({ tagTypeId, tagId })))
    }

    save(medium.id, addTagTagTypeIDs, removeTagTagTypeIDs).then(
      () => {
        close?.()
      },
      e => {
        console.error('Error updating medium\n', e)
      }
    )
  }, [ save, medium, addingTags, removingTags, close ])

  return (
    <Stack>
      <MediumItemMetadataHeader title="タグ">
        <LoadingButton onClick={handleClickSubmit} loading={loading}>
          <span>保存</span>
        </LoadingButton>
        <Button onClick={handleClickCancel}>
          キャンセル
        </Button>
      </MediumItemMetadataHeader>
      <Stack spacing={4}>
        {groups.map(({ type, tags }) => (
          <MediumItemMetadataTagGroupEdit
            key={`${type.id}-${addingTags.get(type.id)?.length ?? 0}`}
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
            key={`${type.id}-${addingTags.get(type.id)?.length ?? 0}`}
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
            includeInputInList
            focus={focus && groups.length === 0}
            placeholder="タイプの追加..."
            disabled={loading}
            value={newTagType}
            getOptionDisabled={({ id }) => groups.some(group => group.type.id === id) || addingTagTypes.some(type => type.id === id)}
            icon={({ ...props }) => <LabelIcon {...props} />}
            onChange={handleChangeNewTagType}
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
