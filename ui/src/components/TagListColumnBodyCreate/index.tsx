'use client'

import type { ChangeEvent, FunctionComponent, SyntheticEvent } from 'react'
import { useCallback, useState } from 'react'
import { useCollator } from '@react-aria/i18n'
import historykana from 'historykana'
import Autocomplete from '@mui/material/Autocomplete'
import Button from '@mui/material/Button'
import Chip from '@mui/material/Chip'
import LoadingButton from '@mui/lab/LoadingButton'
import Portal from '@mui/material/Portal'
import Snackbar from '@mui/material/Snackbar'
import Stack from '@mui/material/Stack'
import TextField from '@mui/material/TextField'

import TagBreadcrumbsList from '@/components/TagBreadcrumbsList'
import { useBeforeUnload, useCreateTag } from '@/hooks'
import type { Tag } from '@/types'

import styles from './styles.module.scss'

const extractKana = (history: string[]): string => {
  return historykana(history, { kanaRegexp: /^[ 　ぁ-ゔー]*[nｎ]?$/ }).replace(/[nｎ]$/, 'ん')
}

const hasChanges = (tag: TagCreate) => tag.name.length > 0 || tag.kana.length > 0 || tag.aliases.length > 0

const TagListColumnBodyCreate: FunctionComponent<TagListColumnBodyCreateProps> = ({
  parent,
  close,
  onCreating,
  onCreate,
}) => {
  const [ createTag, { error, loading } ] = useCreateTag()
  const collator = useCollator()

  const ref = useCallback((input: HTMLElement) => {
    input?.focus({
      preventScroll: true,
    })
  }, [])

  const [ tag, setTag ] = useState<TagCreate>({
    name: '',
    kana: '',
    aliases: [],
  })

  const [ kanaChanged, setKanaChanged ] = useState(false)
  const [ nameHistory, setNameHistory ] = useState<string[]>([])

  const handleChangeName = useCallback((e: ChangeEvent<HTMLInputElement>) => {
    const name = e.currentTarget.value

    const newKanaChanged = name ? kanaChanged : false
    setKanaChanged(newKanaChanged)

    const newNameHistory = name ? [ ...nameHistory, name ] : []
    setNameHistory(newNameHistory)

    const kana = newKanaChanged ? tag.kana : extractKana(newNameHistory)
    setTag(tag => ({
      ...tag,
      name,
      kana,
    }))
  }, [ tag, nameHistory, kanaChanged ])

  const handleChangeKana = useCallback((e: ChangeEvent<HTMLInputElement>) => {
    const kana = e.currentTarget.value
    setKanaChanged(true)
    setTag(tag => ({
      ...tag,
      kana,
    }))
  }, [])

  const handleChangeAliases = useCallback((_e: SyntheticEvent, value: string[]) => {
    const aliases = value.toSorted(collator.compare)
    setTag(tag => ({
      ...tag,
      aliases,
    }))
  }, [ collator ])

  const handleClickCancel = useCallback(() => {
    close()
  }, [ close ])

  const handleClickSubmit = useCallback(() => {
    onCreating?.()
    createTag({
      name: tag.name,
      kana: tag.kana,
      aliases: tag.aliases,
      parentID: parent?.id ?? null,
    }).then(
      newTag => {
        close()
        onCreate?.(newTag)
      },
      e => {
        console.error('Error creating tag\n', e)
      },
    )
  }, [ tag, parent, onCreating, onCreate, createTag, close ])

  const changed = hasChanges(tag)
  useBeforeUnload(changed)

  return (
    <Stack className={styles.container} direction="column-reverse" justifyContent="flex-end">
      <Stack>
        <TextField
          margin="normal"
          label="タイトル"
          disabled={loading}
          value={tag.name}
          onChange={handleChangeName}
          inputRef={ref}
        />
        <TextField
          margin="normal"
          label="ふりがな"
          disabled={loading}
          value={tag.kana}
          onChange={handleChangeKana}
        />
        <Autocomplete
          options={[]}
          disabled={loading}
          value={tag.aliases}
          multiple
          freeSolo
          autoSelect
          disableClearable
          renderInput={params => (
            <TextField
              {...params}
              margin="normal"
              label="別名"
            />
          )}
          onChange={handleChangeAliases}
        />
      </Stack>
      <Stack spacing={1} direction="row" justifyContent={parent ? 'space-between' : 'end'} alignItems="center">
        <Stack className={styles.breadcrumbs} spacing={1} direction="row">
          {parent ? (
            <TagBreadcrumbsList id={parent.id} root noWrap />
          ) : (
            <TagBreadcrumbsList root noWrap />
          )}
        </Stack>
        <Stack className={styles.buttons} spacing={1} direction="row-reverse">
          <LoadingButton onClick={handleClickSubmit} loading={loading} disabled={!changed}>
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
            message="タグを保存できませんでした"
          />
        </Portal>
      ) : null}
    </Stack>
  )
}

export interface TagListColumnBodyCreateProps {
  parent: Tag | null
  close: () => void
  onCreating?: () => void
  onCreate?: (tag: Tag) => void
}

type TagCreate = Omit<Tag, 'id' | 'parent' | 'children'>

export default TagListColumnBodyCreate
