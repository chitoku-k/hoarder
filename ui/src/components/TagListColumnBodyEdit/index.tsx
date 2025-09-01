'use client'

import type { ChangeEvent, FunctionComponent, SyntheticEvent } from 'react'
import { useCallback, useState } from 'react'
import { useCollator } from '@react-aria/i18n'
import Autocomplete from '@mui/material/Autocomplete'
import Button from '@mui/material/Button'
import Portal from '@mui/material/Portal'
import Snackbar from '@mui/material/Snackbar'
import Stack from '@mui/material/Stack'
import TextField from '@mui/material/TextField'

import TagBreadcrumbsList from '@/components/TagBreadcrumbsList'
import TagMoveDialog from '@/components/TagMoveDialog'
import { useBeforeUnload, useUpdateTag } from '@/hooks'
import type { Tag } from '@/types'

import styles from './styles.module.scss'

const hasChanges = (a: Tag, b: Tag) => {
  if (a.name !== b.name || a.kana !== b.kana || a.aliases.length !== b.aliases.length) {
    return true
  }

  for (const [ idx, alias ] of a.aliases.entries()) {
    if (alias !== b.aliases[idx]) {
      return true
    }
  }

  return false
}

const TagListColumnBodyEdit: FunctionComponent<TagListColumnBodyEditProps> = ({
  tag: current,
  close,
  onMove,
}) => {
  const [ updateTag, { error, loading } ] = useUpdateTag()
  const collator = useCollator()

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
    // eslint-disable-next-line @typescript-eslint/unbound-method
    const aliases = value.toSorted(collator.compare)
    setTag(tag => ({
      ...tag,
      aliases,
    }))
  }, [ collator ])

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

  const changed = hasChanges(tag, current)
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
          onChange={handleChangeAliases}
        />
      </Stack>
      <Stack spacing={1} direction="row" justifyContent="space-between" alignItems="center">
        <Stack className={styles.breadcrumbs} spacing={1} direction="row">
          <TagBreadcrumbsList id={tag.id} parent root noWrap />
        </Stack>
        <Stack className={styles.buttons} spacing={1} direction="row-reverse">
          <Button onClick={handleClickSubmit} loading={loading}>
            保存
          </Button>
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
