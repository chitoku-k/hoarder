'use client'

import type { FunctionComponent, SyntheticEvent } from 'react'
import { useCallback, useState } from 'react'
import Stack from '@mui/material/Stack'
import LabelIcon from '@mui/icons-material/Label'

import AutocompleteTagType from '@/components/AutocompleteTagType'
import MediumItemMetadataHeader from '@/components/MediumItemMetadataHeader'
import MediumItemMetadataTagGroupEdit from '@/components/MediumItemMetadataTagGroupEdit'
import type { TagTagTypeInput } from '@/hooks/types.generated'
import type { Tag, TagType } from '@/types'

const MediumItemMetadataTagCreate: FunctionComponent<MediumItemMetadataTagCreateProps> = ({
  loading,
  setTagTagTypeIDs,
}) => {
  const [ focusedTagType, setFocusedTagType ] = useState<TagType | null>(null)
  const [ newTagType, setNewTagType ] = useState<TagType | null>(null)
  const [ newTagTypeInput, setNewTagTypeInput ] = useState('')

  const [ addingTagTypes, setAddingTagTypes ] = useState<TagType[]>([])
  const [ addingTags, setAddingTags ] = useState(new Map<TagTypeID, Tag[]>())

  const handleChangeNewTagType = useCallback((type: TagType | null) => {
    if (!type) {
      return
    }

    setNewTagType(null)
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
  }, [])

  const restoreTagType = useCallback(() => {}, [])

  const addTag = useCallback((type: TagType, tag: Tag) => {
    setFocusedTagType(type)

    setAddingTags(addingTags => {
      const newTags = addingTags.get(type.id) ?? []
      if (newTags.some(({ id }) => id === tag.id)) {
        return addingTags
      }

      newTags.push(tag)

      const newAddingTags = new Map(addingTags.set(type.id, newTags))
      setTagTagTypeIDs(() => resolveTagTagTypeIDs(newAddingTags))
      return newAddingTags
    })
  }, [ setTagTagTypeIDs ])

  const removeTag = useCallback((type: TagType, tag: Tag) => {
    setFocusedTagType(null)

    setAddingTags(addingTags => {
      const newTags = addingTags.get(type.id) ?? []
      const idx = newTags.findIndex(({ id }) => id === tag.id)
      if (idx < 0) {
        return addingTags
      }

      const newAddingTags = new Map(addingTags.set(type.id, newTags.toSpliced(idx, 1)))
      setTagTagTypeIDs(() => resolveTagTagTypeIDs(newAddingTags))
      return newAddingTags
    })
  }, [ setTagTagTypeIDs ])

  const restoreTag = useCallback(() => {}, [])

  const resolveTagTagTypeIDs = (addingTags: Map<TagTypeID, Tag[]>) => {
    const addTagTagTypeIDs: TagTagTypeInput[] = []
    for (const [ tagTypeId, tags ] of addingTags) {
      addTagTagTypeIDs.push(...tags.map(({ id: tagId }) => ({ tagTypeId, tagId })))
    }
    return addTagTagTypeIDs
  }

  return (
    <Stack>
      <MediumItemMetadataHeader title="タグ" />
      <Stack spacing={4}>
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
            removingTags={[]}
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
            placeholder="タイプの追加..."
            disabled={loading}
            value={newTagType}
            inputValue={newTagTypeInput}
            getOptionDisabled={({ id }) => addingTagTypes.some(type => type.id === id)}
            icon={({ ...props }) => <LabelIcon {...props} />}
            onChange={handleChangeNewTagType}
            onInputChange={handleChangeNewTagTypeInput}
          />
        </Stack>
      </Stack>
    </Stack>
  )
}

export interface MediumItemMetadataTagCreateProps {
  loading: boolean
  setTagTagTypeIDs: (setTagTagTypeIDs: () => TagTagTypeInput[]) => void
}

type TagTypeID = string

export default MediumItemMetadataTagCreate
