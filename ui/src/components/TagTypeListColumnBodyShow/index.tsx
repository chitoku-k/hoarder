'use client'

import type { FunctionComponent } from 'react'
import { useCallback } from 'react'
import Button from '@mui/material/Button'
import Stack from '@mui/material/Stack'
import TextField from '@mui/material/TextField'

import type { TagType } from '@/types'

import styles from './styles.module.scss'

const TagTypeListColumnBodyShow: FunctionComponent<TagTypeListColumnBodyShowProps> = ({
  tagType,
  edit,
}) => {
  const handleClickEdit = useCallback(() => {
    edit(tagType)
  }, [ tagType, edit ])

  return (
    <Stack className={styles.container} direction="column-reverse" justifyContent="flex-end">
      <Stack>
        <TextField
          margin="normal"
          label="タイトル"
          value={tagType.name}
          onDoubleClick={handleClickEdit}
          slotProps={{
            htmlInput: {
              readOnly: true,
            },
          }}
        />
        <TextField
          margin="normal"
          label="ふりがな"
          value={tagType.kana}
          onDoubleClick={handleClickEdit}
        />
        <TextField
          margin="normal"
          label="スラッグ"
          value={tagType.slug}
          onDoubleClick={handleClickEdit}
        />
      </Stack>
      <Stack direction="row" justifyContent="flex-end">
        <Stack spacing={1} direction="row-reverse">
          <Button onClick={handleClickEdit}>
            <span>編集</span>
          </Button>
        </Stack>
      </Stack>
    </Stack>
  )
}

export interface TagTypeListColumnBodyShowProps {
  readonly tagType: TagType
  readonly edit: (tagType: TagType) => void
}

export default TagTypeListColumnBodyShow
