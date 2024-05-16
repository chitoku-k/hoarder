'use client'

import type { ChangeEvent, FunctionComponent } from 'react'
import { useCallback, useState } from 'react'
import Button from '@mui/material/Button'
import LoadingButton from '@mui/lab/LoadingButton'
import Portal from '@mui/material/Portal'
import Snackbar from '@mui/material/Snackbar'
import Stack from '@mui/material/Stack'
import TextField from '@mui/material/TextField'

import { useUpdateExternalService } from '@/hooks'
import type { ExternalService } from '@/types'

import styles from './styles.module.scss'

const ExternalServiceListColumnBodyEdit: FunctionComponent<ExternalServiceListColumnBodyEditProps> = ({
  externalService: current,
  close,
  onEdit,
}) => {
  const [ updateExternalService, { error, loading } ] = useUpdateExternalService()

  const ref = useCallback((input: HTMLElement) => {
    input?.focus({
      preventScroll: true,
    })
  }, [])

  const [ externalService, setExternalService ] = useState(current)

  const handleChangeName = useCallback((e: ChangeEvent<HTMLInputElement>) => {
    const name = e.currentTarget.value
    setExternalService(externalService => ({
      ...externalService,
      name,
    }))
  }, [])

  const handleChangeSlug = useCallback((e: ChangeEvent<HTMLInputElement>) => {
    const slug = e.currentTarget.value
    setExternalService(externalService => ({
      ...externalService,
      slug,
    }))
  }, [])

  const handleChangeBaseUrl = useCallback((e: ChangeEvent<HTMLInputElement>) => {
    const baseUrl = e.currentTarget.value
    setExternalService(externalService => ({
      ...externalService,
      baseUrl,
    }))
  }, [])

  const handleClickCancel = useCallback(() => {
    close()
  }, [ close ])

  const handleClickSubmit = useCallback(() => {
    updateExternalService({
      id: externalService.id,
      slug: externalService.slug,
      name: externalService.name,
      baseUrl: externalService.baseUrl,
    }).then(
      newExternalService => {
        close()
        onEdit(newExternalService)
      },
      e => {
        console.error('Error updating external service\n', e)
      },
    )
  }, [ externalService, updateExternalService, onEdit, close ])

  return (
    <Stack className={styles.container} direction="column-reverse" justifyContent="flex-end">
      <Stack>
        <TextField
          margin="normal"
          label="タイトル"
          disabled={loading}
          value={externalService.name}
          onChange={handleChangeName}
          inputRef={ref}
        />
        <TextField
          margin="normal"
          label="スラッグ"
          disabled={loading}
          value={externalService.slug}
          onChange={handleChangeSlug}
        />
        <TextField
          margin="normal"
          label="種別（変更不可）"
          disabled={loading}
          value={externalService.kind}
          inputProps={{
            readOnly: true,
            tabIndex: -1,
          }}
        />
        <TextField
          margin="normal"
          label="ベース URL"
          disabled={loading}
          value={externalService.baseUrl}
          onChange={handleChangeBaseUrl}
        />
      </Stack>
      <Stack direction="row" justifyContent="flex-end">
        <Stack className={styles.buttons} spacing={1} direction="row-reverse">
          <LoadingButton onClick={handleClickSubmit} loading={loading}>
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
            message="サービスを保存できませんでした"
          />
        </Portal>
      ) : null}
    </Stack>
  )
}

export interface ExternalServiceListColumnBodyEditProps {
  externalService: ExternalService
  close: () => void
  onEdit: (externalService: ExternalService) => void
}

export default ExternalServiceListColumnBodyEdit
