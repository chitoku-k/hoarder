'use client'

import type { ChangeEvent, FunctionComponent } from 'react'
import { useCallback, useState } from 'react'
import Button from '@mui/material/Button'
import LoadingButton from '@mui/lab/LoadingButton'
import Portal from '@mui/material/Portal'
import Snackbar from '@mui/material/Snackbar'
import Stack from '@mui/material/Stack'
import TextField from '@mui/material/TextField'

import { TAG_TYPE_SLUG_DUPLICATE, useError, useUpdateTagType } from '@/hooks'
import type { TagType } from '@/types'

import styles from './styles.module.scss'

const TagTypeListColumnBodyEdit: FunctionComponent<TagTypeListColumnBodyEditProps> = ({
  tagType: current,
  close,
  onEdit,
}) => {
  const [ updateTagType, { error, loading } ] = useUpdateTagType()
  const { graphQLError } = useError()

  const ref = useCallback((input: HTMLElement) => {
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
    }).then(
      newTagType => {
        close()
        onEdit(newTagType)
      },
      e => {
        console.error('Error updating tag type\n', e)
      },
    )
  }, [ tagType, updateTagType, onEdit, close ])

  const tagTypeSlugDuplicate = graphQLError(error, TAG_TYPE_SLUG_DUPLICATE)
  const isSlugDuplicate = tagTypeSlugDuplicate?.extensions.details.data.slug === tagType.slug

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
          <LoadingButton onClick={handleClickSubmit} loading={loading} disabled={isSlugDuplicate}>
            <span>保存</span>
          </LoadingButton>
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
