'use client'

import type { ChangeEvent, FunctionComponent } from 'react'
import { useCallback, useState } from 'react'
import Button from '@mui/material/Button'
import Portal from '@mui/material/Portal'
import Snackbar from '@mui/material/Snackbar'
import Stack from '@mui/material/Stack'
import TextField from '@mui/material/TextField'

import { TAG_TYPE_SLUG_DUPLICATE, useBeforeUnload, useError, useUpdateTagType } from '@/hooks'
import type { TagType } from '@/types'

import styles from './styles.module.scss'

const hasChanges = (a: TagType, b: TagType) => a.name !== b.name || a.kana !== b.kana

const TagTypeListColumnBodyEdit: FunctionComponent<TagTypeListColumnBodyEditProps> = ({
  tagType: current,
  close,
  onEdit,
}) => {
  const [ updateTagType, { error, loading } ] = useUpdateTagType()
  const { graphQLError } = useError()

  const ref = useCallback((input: HTMLElement | null) => {
    input?.focus({
      preventScroll: true,
    })
  }, [])

  const [ tagType, setTagType ] = useState(current)

  const handleChangeName = useCallback((e: ChangeEvent<HTMLInputElement>) => {
    const name = e.currentTarget.value
    setTagType(tagType => ({
      ...tagType,
      name,
    }))
  }, [])

  const handleChangeKana = useCallback((e: ChangeEvent<HTMLInputElement>) => {
    const kana = e.currentTarget.value
    setTagType(tagType => ({
      ...tagType,
      kana,
    }))
  }, [])

  const handleChangeSlug = useCallback((e: ChangeEvent<HTMLInputElement>) => {
    const slug = e.currentTarget.value
    setTagType(tagType => ({
      ...tagType,
      slug,
    }))
  }, [])

  const handleClickCancel = useCallback(() => {
    close()
  }, [ close ])

  const handleClickSubmit = useCallback(() => {
    updateTagType({
      id: tagType.id,
      slug: tagType.slug,
      name: tagType.name,
      kana: tagType.kana,
    }).then(
      newTagType => {
        close()
        onEdit(newTagType)
      },
      (e: unknown) => {
        console.error('Error updating tag type\n', e)
      },
    )
  }, [ tagType, updateTagType, onEdit, close ])

  const tagTypeSlugDuplicate = graphQLError(error, TAG_TYPE_SLUG_DUPLICATE)
  const isSlugDuplicate = tagTypeSlugDuplicate?.extensions.details.data.slug === tagType.slug
  const changed = hasChanges(tagType, current)
  useBeforeUnload(changed)

  return (
    <Stack className={styles.container} direction="column-reverse" justifyContent="flex-end">
      <Stack>
        <TextField
          margin="normal"
          label="タイトル"
          disabled={loading}
          value={tagType.name}
          onChange={handleChangeName}
          inputRef={ref}
        />
        <TextField
          margin="normal"
          label="ふりがな"
          disabled={loading}
          value={tagType.kana}
          onChange={handleChangeKana}
        />
        {isSlugDuplicate ? (
          <TextField
            error
            margin="normal"
            label="スラッグ"
            helperText="このスラッグはすでに使われています"
            disabled={loading}
            value={tagType.slug}
            onChange={handleChangeSlug}
          />
        ) : (
          <TextField
            margin="normal"
            label="スラッグ"
            disabled={loading}
            value={tagType.slug}
            onChange={handleChangeSlug}
          />
        )}
      </Stack>
      <Stack direction="row" justifyContent="flex-end">
        <Stack className={styles.buttons} spacing={1} direction="row-reverse">
          <Button onClick={handleClickSubmit} loading={loading} disabled={isSlugDuplicate}>
            保存
          </Button>
          <Button onClick={handleClickCancel}>
            キャンセル
          </Button>
        </Stack>
      </Stack>
      {error ? (
        <Portal>
          <Snackbar
            open
            anchorOrigin={{ vertical: 'top', horizontal: 'center' }}
            message="タイプを保存できませんでした"
          />
        </Portal>
      ) : null}
    </Stack>
  )
}

export interface TagTypeListColumnBodyEditProps {
  tagType: TagType
  close: () => void
  onEdit: (tagType: TagType) => void
}

export default TagTypeListColumnBodyEdit
