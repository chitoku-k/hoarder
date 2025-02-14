'use client'

import type { FunctionComponent } from 'react'
import { useCallback } from 'react'
import Button from '@mui/material/Button'
import Stack from '@mui/material/Stack'
import TextField from '@mui/material/TextField'

import type { ExternalService } from '@/types'

import styles from './styles.module.scss'

const ExternalServiceListColumnBodyShow: FunctionComponent<ExternalServiceListColumnBodyShowProps> = ({
  externalService,
  edit,
}) => {
  const handleClickEdit = useCallback(() => {
    edit(externalService)
  }, [ externalService, edit ])

  return (
    <Stack className={styles.container} direction="column-reverse" justifyContent="flex-end">
      <Stack>
        <TextField
          margin="normal"
          label="タイトル"
          value={externalService.name}
          onDoubleClick={handleClickEdit}
          slotProps={{
            htmlInput: {
              readOnly: true,
            },
          }}
        />
        <TextField
          margin="normal"
          label="スラッグ"
          value={externalService.slug}
          onDoubleClick={handleClickEdit}
          slotProps={{
            htmlInput: {
              readOnly: true,
            },
          }}
        />
        <TextField
          margin="normal"
          label="種別（変更不可）"
          value={externalService.kind}
          onDoubleClick={handleClickEdit}
          slotProps={{
            htmlInput: {
              readOnly: true,
            },
          }}
        />
        <TextField
          margin="normal"
          label="ベース URL"
          value={externalService.baseUrl ?? ''}
          onDoubleClick={handleClickEdit}
          slotProps={{
            htmlInput: {
              readOnly: true,
            },
          }}
        />
        <TextField
          margin="normal"
          label="URL 正規表現"
          value={externalService.urlPattern ?? ''}
          onDoubleClick={handleClickEdit}
          slotProps={{
            htmlInput: {
              readOnly: true,
            },
          }}
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

export interface ExternalServiceListColumnBodyShowProps {
  externalService: ExternalService
  edit: (externalService: ExternalService) => void
}

export default ExternalServiceListColumnBodyShow
