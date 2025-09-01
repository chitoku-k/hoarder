'use client'

import type { ComponentPropsWithoutRef, FunctionComponent, SyntheticEvent } from 'react'
import { useCallback, useState } from 'react'
import Stack from '@mui/material/Stack'
import LabelIcon from '@mui/icons-material/Label'

import AutocompleteTagType from '@/components/AutocompleteTagType'
import MediumItemMetadataHeader from '@/components/MediumItemMetadataHeader'
import MediumItemMetadataTagGroupEdit from '@/components/MediumItemMetadataTagGroupEdit'
import type { TagTagTypeInput } from '@/graphql/types.generated'
import { useBeforeUnload } from '@/hooks'
import type { Tag, TagType } from '@/types'

import styles from './styles.module.scss'

const hasChanges = (addingTagTypes: TagType[], addingTags: Map<TagTypeID, Tag[]>) => {
  if (addingTagTypes.length > 0) {
    return true
  }

  for (const tags of addingTags.values()) {
    if (tags.length > 0) {
      return true
    }
  }

  return false
}

const MediumItemMetadataTagCreate: FunctionComponent<MediumItemMetadataTagCreateProps> = ({
  loading,
  setTagTagTypeIDs,
}) => {
  const [ focusedTagType, setFocusedTagType ] = useState<TagType | null>(null)
  const [ newTagTypeInput, setNewTagTypeInput ] = useState('')

  const [ addingTagTypes, setAddingTagTypes ] = useState<TagType[]>([])
  const [ addingTags, setAddingTags ] = useState(new Map<TagTypeID, Tag[]>())

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
  }, [])

  const restoreTagType = useCallback(() => {}, [])

  const addTag = useCallback((type: TagType, tag: Tag) => {
    setFocusedTagType(type)

    const newTags = addingTags.get(type.id) ?? []
    if (newTags.some(({ id }) => id === tag.id)) {
      return
    }

    const newAddingTags = new Map(addingTags).set(type.id, [ ...newTags, tag ])
    setAddingTags(newAddingTags)
    setTagTagTypeIDs(() => resolveTagTagTypeIDs(newAddingTags))
  }, [ addingTags, setTagTagTypeIDs ])

  const removeTag = useCallback((type: TagType, tag: Tag) => {
    setFocusedTagType(null)

    const newTags = addingTags.get(type.id) ?? []
    const idx = newTags.findIndex(({ id }) => id === tag.id)
    if (idx < 0) {
      return
    }

    const newAddingTags = new Map(addingTags).set(type.id, newTags.toSpliced(idx, 1))
    setAddingTags(newAddingTags)
    setTagTagTypeIDs(() => resolveTagTagTypeIDs(newAddingTags))
  }, [ addingTags, setTagTagTypeIDs ])

  const restoreTag = useCallback(() => {}, [])

  const resolveTagTagTypeIDs = (addingTags: Map<TagTypeID, Tag[]>) => {
    const addTagTagTypeIDs: TagTagTypeInput[] = []
    for (const [ tagTypeId, tags ] of addingTags) {
      addTagTagTypeIDs.push(...tags.map(({ id: tagId }) => ({ tagTypeId, tagId })))
    }
    return addTagTagTypeIDs
  }

  const renderTagTypeOption = useCallback(({ key, ...props }: ComponentPropsWithoutRef<'li'>, option: TagType) => (
    <li key={key} {...props}>
      <Stack direction="row" spacing={0.5} alignItems="start">
        <LabelIcon className={styles.tagTypeSearchIcon} fontSize="small" />
        <span className={styles.tagTypeSearchText}>{option.name}</span>
      </Stack>
    </li>
  ), [])

  const changed = hasChanges(addingTagTypes, addingTags)
  useBeforeUnload(changed)

  return (
    <Stack>
      <MediumItemMetadataHeader title="タグ" />
      <Stack spacing={4}>
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
            renderOption={renderTagTypeOption}
            value={null}
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
