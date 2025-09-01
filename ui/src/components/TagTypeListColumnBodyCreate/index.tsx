'use client'

import type { ChangeEvent, FunctionComponent } from 'react'
import { useCallback, useState } from 'react'
import Button from '@mui/material/Button'
import Portal from '@mui/material/Portal'
import Snackbar from '@mui/material/Snackbar'
import Stack from '@mui/material/Stack'
import TextField from '@mui/material/TextField'

import { TAG_TYPE_SLUG_DUPLICATE, useBeforeUnload, useCreateTagType, useError, useHistorykana } from '@/hooks'
import type { TagType } from '@/types'

import styles from './styles.module.scss'

const hasChanges = (tagType: TagTypeCreate) => tagType.name.length > 0 || tagType.slug.length > 0 || tagType.kana.length > 0

const TagTypeListColumnBodyCreate: FunctionComponent<TagTypeListColumnBodyCreateProps> = ({
  close,
}) => {
  const [ createTagType, { error, loading } ] = useCreateTagType()
  const { graphQLError } = useError()
  const extractKana = useHistorykana()

  const ref = useCallback((input: HTMLElement | null) => {
    input?.focus({
      preventScroll: true,
    })
  }, [])

  const [ tagType, setTagType ] = useState<TagTypeCreate>({
    name: '',
    slug: '',
    kana: '',
  })

  const [ kanaChanged, setKanaChanged ] = useState(false)
  const [ nameHistory, setNameHistory ] = useState<string[]>([])

  const handleChangeName = useCallback((e: ChangeEvent<HTMLInputElement>) => {
    const name = e.currentTarget.value

    const newKanaChanged = name ? kanaChanged : false
    setKanaChanged(newKanaChanged)

    const newNameHistory = name ? [ ...nameHistory, name ] : []
    setNameHistory(newNameHistory)

    const kana = newKanaChanged ? tagType.kana : extractKana(newNameHistory)
    setTagType(tagType => ({
      ...tagType,
      name,
      kana,
    }))
  }, [ tagType, nameHistory, kanaChanged ])

  const handleChangeKana = useCallback((e: ChangeEvent<HTMLInputElement>) => {
    const kana = e.currentTarget.value
    setKanaChanged(true)
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
    createTagType({
      slug: tagType.slug,
      name: tagType.name,
      kana: tagType.kana,
    }).then(
      () => {
        close()
      },
      (e: unknown) => {
        console.error('Error creating tag type\n', e)
      },
    )
  }, [ tagType, createTagType, close ])

  const tagTypeSlugDuplicate = graphQLError(error, TAG_TYPE_SLUG_DUPLICATE)
  const isSlugDuplicate = tagTypeSlugDuplicate?.extensions.details.data.slug === tagType.slug
  const changed = hasChanges(tagType)
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
          <Button onClick={handleClickSubmit} loading={loading} disabled={!changed || isSlugDuplicate}>
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

export interface TagTypeListColumnBodyCreateProps {
  close: () => void
}

type TagTypeCreate = Omit<TagType, 'id'>

export default TagTypeListColumnBodyCreate
