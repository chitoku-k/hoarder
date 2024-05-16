'use client'

import type { ChangeEvent, FunctionComponent, SyntheticEvent } from 'react'
import { useCallback, useState } from 'react'
import Autocomplete from '@mui/material/Autocomplete'
import Button from '@mui/material/Button'
import Chip from '@mui/material/Chip'
import LoadingButton from '@mui/lab/LoadingButton'
import Portal from '@mui/material/Portal'
import Snackbar from '@mui/material/Snackbar'
import Stack from '@mui/material/Stack'
import TextField from '@mui/material/TextField'

import TagBreadcrumbsList from '@/components/TagBreadcrumbsList'
import TagMoveDialog from '@/components/TagMoveDialog'
import { useUpdateTag } from '@/hooks'
import type { Tag } from '@/types'

import styles from './styles.module.scss'

const TagListColumnBodyEdit: FunctionComponent<TagListColumnBodyEditProps> = ({
  tag: current,
  close,
  onMove,
}) => {
  const [ updateTag, { error, loading } ] = useUpdateTag()

  const ref = useCallback((input: HTMLElement) => {
    input?.focus({
      preventScroll: true,
    })
  }, [])

  const [ movingTag, setMovingTag ] = useState(false)
  const [ tag, setTag ] = useState(current)

  const handleChangeName = useCallback((e: ChangeEvent<HTMLInputElement>) => {
    const name = e.currentTarget.value
    setTag(tag => ({
      ...tag,
      name,
    }))
  }, [])

  const handleChangeKana = useCallback((e: ChangeEvent<HTMLInputElement>) => {
    const kana = e.currentTarget.value
    setTag(tag => ({
      ...tag,
      kana,
    }))
  }, [])

  const handleChangeAliases = useCallback((_e: SyntheticEvent, value: string[]) => {
    const aliases = value.toSorted()
    setTag(tag => ({
      ...tag,
      aliases,
    }))
  }, [])

  const handleClickMoveTag = useCallback(() => {
    setMovingTag(true)
  }, [])

  const closeMoveTag = useCallback(() => {
    setMovingTag(false)
  }, [])

  const handleClickCancel = useCallback(() => {
    close()
  }, [ close ])

  const handleClickSubmit = useCallback(() => {
    const addAliases = tag.aliases.filter(alias => !current.aliases.includes(alias))
    const removeAliases = current.aliases.filter(alias => !tag.aliases.includes(alias))

    updateTag({
      id: tag.id,
      name: tag.name,
      kana: tag.kana,
      addAliases,
      removeAliases,
    }).then(
      () => {
        close()
      },
      e => {
        console.error('Error updating tag\n', e)
      },
    )
  }, [ tag, current, updateTag, close ])

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
          options={current.aliases.filter(alias => !tag.aliases.includes(alias))}
          disabled={loading}
          value={tag.aliases}
          multiple
          freeSolo
          autoSelect
          disableClearable
          renderInput={({ ...params }) => (
            <TextField
              {...params}
              margin="normal"
              label="別名"
            />
          )}
          renderTags={(value, getCustomizedTagProps) => value.map((option, index) => {
            const { key, ...props } = getCustomizedTagProps({ index })
            return (
              <Chip key={key} label={option} size="medium" {...props} />
            )
          })}
          onChange={handleChangeAliases}
        />
      </Stack>
      <Stack spacing={1} direction="row" justifyContent="space-between" alignItems="center">
        <Stack className={styles.breadcrumbs} spacing={1} direction="row">
          <TagBreadcrumbsList id={tag.id} parent root noWrap />
        </Stack>
        <Stack className={styles.buttons} spacing={1} direction="row-reverse">
          <LoadingButton onClick={handleClickSubmit} loading={loading}>
            <span>保存</span>
          </LoadingButton>
          <Button onClick={handleClickCancel}>
            キャンセル
          </Button>
          <Button onClick={handleClickMoveTag}>
            移動
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
      {movingTag ? (
        <TagMoveDialog
          key={tag.id}
          tag={tag}
          close={closeMoveTag}
          onMove={onMove}
        />
      ) : null}
    </Stack>
  )
}

export interface TagListColumnBodyEditProps {
  tag: Tag
  close: () => void
  onMove: (tag: Tag) => void
}

export default TagListColumnBodyEdit
