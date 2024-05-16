'use client'

import type { ChangeEvent, FunctionComponent } from 'react'
import { useCallback, useState } from 'react'
import Button from '@mui/material/Button'
import LoadingButton from '@mui/lab/LoadingButton'
import Portal from '@mui/material/Portal'
import Snackbar from '@mui/material/Snackbar'
import Stack from '@mui/material/Stack'
import TextField from '@mui/material/TextField'

import { EXTERNAL_SERVICE_SLUG_DUPLICATE, useCreateExternalService, useError } from '@/hooks'
import type { ExternalService } from '@/types'

import styles from './styles.module.scss'

const ExternalServiceListColumnBodyCreate: FunctionComponent<ExternalServiceListColumnBodyCreateProps> = ({
  close,
}) => {
  const [ createExternalService, { error, loading } ] = useCreateExternalService()
  const { graphQLError } = useError()

  const ref = useCallback((input: HTMLElement) => {
    input?.focus({
      preventScroll: true,
    })
  }, [])

  const [ externalService, setExternalService ] = useState<Omit<ExternalService, 'id'>>({
    name: '',
    slug: '',
    kind: '',
    baseUrl: '',
  })

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

  const handleChangeKind = useCallback((e: ChangeEvent<HTMLInputElement>) => {
    const kind = e.currentTarget.value
    setExternalService(externalService => ({
      ...externalService,
      kind,
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
    createExternalService(externalService).then(
      () => {
        close()
      },
      e => {
        console.error('Error creating external service\n', e)
      },
    )
  }, [ externalService, createExternalService, close ])

  const externalServiceSlugDuplicate = graphQLError(error, EXTERNAL_SERVICE_SLUG_DUPLICATE)
  const isSlugDuplicate = externalServiceSlugDuplicate?.extensions.details.data.slug === externalService.slug
  const empty = externalService.name.length === 0 || externalService.slug.length == 0

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
        {isSlugDuplicate ? (
          <TextField
            error
            margin="normal"
            label="スラッグ"
            helperText="このスラッグはすでに使われています"
            disabled={loading}
            value={externalService.slug}
            onChange={handleChangeSlug}
          />
        ) : (
          <TextField
            margin="normal"
            label="スラッグ"
            disabled={loading}
            value={externalService.slug}
            onChange={handleChangeSlug}
          />
        )}
        <TextField
          margin="normal"
          label="種別（変更不可）"
          disabled={loading}
          value={externalService.kind}
          onChange={handleChangeKind}
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
            message="サービスを保存できませんでした"
          />
        </Portal>
      ) : null}
    </Stack>
  )
}

export interface ExternalServiceListColumnBodyCreateProps {
  close: () => void
}

export default ExternalServiceListColumnBodyCreate
