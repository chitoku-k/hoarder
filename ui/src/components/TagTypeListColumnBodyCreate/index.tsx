'use client'

import type { ChangeEvent, FunctionComponent } from 'react'
import { useCallback, useState } from 'react'
import Button from '@mui/material/Button'
import LoadingButton from '@mui/lab/LoadingButton'
import Portal from '@mui/material/Portal'
import Snackbar from '@mui/material/Snackbar'
import Stack from '@mui/material/Stack'
import TextField from '@mui/material/TextField'

import { TAG_TYPE_SLUG_DUPLICATE, useError, useCreateTagType } from '@/hooks'
import type { TagType } from '@/types'

import styles from './styles.module.scss'

const TagTypeListColumnBodyCreate: FunctionComponent<TagTypeListColumnBodyCreateProps> = ({
  close,
}) => {
  const [ createTagType, { error, loading } ] = useCreateTagType()
  const { graphQLError } = useError()

  const ref = useCallback((input: HTMLElement) => {
    input?.focus({
      preventScroll: true,
    })
  }, [])

  const [ tagType, setTagType ] = useState<Omit<TagType, 'id'>>({
    name: '',
    slug: '',
  })

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
    createTagType({
      name: tagType.name,
      slug: tagType.slug,
    }).then(
      () => {
        close()
      },
      e => {
        console.error('Error creating tag type\n', e)
      },
    )
  }, [ tagType, createTagType, close ])

  const tagTypeSlugDuplicate = graphQLError(error, TAG_TYPE_SLUG_DUPLICATE)
  const isSlugDuplicate = tagTypeSlugDuplicate?.extensions.details.data.slug === tagType.slug
  const empty = tagType.name.length === 0 || tagType.slug.length == 0

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
          <LoadingButton onClick={handleClickSubmit} loading={loading} disabled={empty || isSlugDuplicate}>
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

export interface TagTypeListColumnBodyCreateProps {
  close: () => void
}

export default TagTypeListColumnBodyCreate
