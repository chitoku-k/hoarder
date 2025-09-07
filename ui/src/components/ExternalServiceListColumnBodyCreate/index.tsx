'use client'

import type { ChangeEvent, FunctionComponent } from 'react'
import { useCallback, useEffect, useRef, useState } from 'react'
import Button from '@mui/material/Button'
import Chip from '@mui/material/Chip'
import Portal from '@mui/material/Portal'
import Snackbar from '@mui/material/Snackbar'
import Stack from '@mui/material/Stack'
import TextField from '@mui/material/TextField'

import { EXTERNAL_SERVICE_SLUG_DUPLICATE, EXTERNAL_SERVICE_URL_PATTERN_INVALID, useCreateExternalService, useError } from '@/hooks'
import type { ExternalService } from '@/types'

import styles from './styles.module.scss'

const ExternalServiceListColumnBodyCreate: FunctionComponent<ExternalServiceListColumnBodyCreateProps> = ({
  close,
}) => {
  const [ createExternalService, { error, loading } ] = useCreateExternalService()
  const { graphQLError } = useError()

  const ref = useCallback((input: HTMLElement | null) => {
    input?.focus({
      preventScroll: true,
    })
  }, [])

  const urlPatternRef = useRef<HTMLInputElement>(null)
  const [ urlPatternSelection, setUrlPatternSelection ] = useState<[ number, number ] | [ null, null ]>([ null, null ])

  const [ externalService, setExternalService ] = useState<Omit<ExternalService, 'id'>>({
    name: '',
    slug: '',
    kind: '',
    baseUrl: '',
    urlPattern: '',
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

  const handleChangeUrlPattern = useCallback((e: ChangeEvent<HTMLInputElement>) => {
    const urlPattern = e.currentTarget.value
    setExternalService(externalService => ({
      ...externalService,
      urlPattern,
    }))
    setUrlPatternSelection([ null, null ])
  }, [])

  const handleClickInsertCaptureGroup = useCallback((name: string) => {
    const { current } = urlPatternRef
    if (!current) {
      return
    }

    const prefix = `(?<${name}>`
    const suffix = ')'
    const selectionStart = current.selectionStart ?? 0
    const selectionEnd = current.selectionEnd ?? 0
    const urlPattern = current.value.slice(0, selectionStart)
      + prefix
      + current.value.slice(selectionStart, selectionEnd)
      + suffix
      + current.value.slice(selectionEnd)

    setExternalService(externalService => ({
      ...externalService,
      urlPattern,
    }))
    setUrlPatternSelection([ selectionStart + prefix.length, selectionEnd + prefix.length ])
  }, [ urlPatternRef ])

  useEffect(() => {
    const { current } = urlPatternRef
    if (!current) {
      return
    }

    const [ selectionStart, selectionEnd ] = urlPatternSelection
    if (selectionStart === null) {
      return
    }

    current.setSelectionRange(selectionStart, selectionEnd)
    current.focus()
  }, [ urlPatternRef, urlPatternSelection ])

  const handleClickCancel = useCallback(() => {
    close()
  }, [ close ])

  const handleClickSubmit = useCallback(() => {
    createExternalService(externalService).then(
      () => {
        close()
      },
      (e: unknown) => {
        console.error('Error creating external service\n', e)
      },
    )
  }, [ externalService, createExternalService, close ])

  const externalServiceSlugDuplicate = graphQLError(error, EXTERNAL_SERVICE_SLUG_DUPLICATE)
  const externalServiceUrlPatternInvalid = graphQLError(error, EXTERNAL_SERVICE_URL_PATTERN_INVALID)
  const externalServiceUrlPatternInvalidDescription = externalServiceUrlPatternInvalid?.extensions.details.data.description
  const isSlugDuplicate = externalServiceSlugDuplicate?.extensions.details.data.slug === externalService.slug
  const isUrlPatternInvalid = externalServiceUrlPatternInvalid?.extensions.details.data.urlPattern === externalService.urlPattern
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
        {isUrlPatternInvalid ? (
          <TextField
            error
            margin="normal"
            label="URL 正規表現"
            helperText={
              <Stack>
                パターンが正しくありません
                {externalServiceUrlPatternInvalidDescription ? (
                  <Stack overflow="auto" whiteSpace="pre">
                    <code>
                      {externalServiceUrlPatternInvalidDescription}
                    </code>
                  </Stack>
                ) : null}
              </Stack>
            }
            slotProps={{
              formHelperText: {
                component: 'div',
              },
            }}
            disabled={loading}
            value={externalService.urlPattern}
            onChange={handleChangeUrlPattern}
          />
        ) : (
          <TextField
            margin="normal"
            label="URL 正規表現"
            helperText={
              <Stack spacing={0.5}>
                パターンに名前付きキャプチャーグループを使用してソースを検索対象にします
                <Stack direction="row" spacing={0.5}>
                  {[ 'id', 'creatorId' ].map(name => (
                    <Chip
                      key={name}
                      label={`(?<${name}>)`}
                      size="small"
                      variant="outlined"
                      onClick={() => handleClickInsertCaptureGroup(name)}
                    />
                  ))}
                </Stack>
              </Stack>
            }
            slotProps={{
              formHelperText: {
                component: 'div',
              },
            }}
            disabled={loading}
            value={externalService.urlPattern}
            onChange={handleChangeUrlPattern}
            inputRef={urlPatternRef}
          />
        )}
      </Stack>
      <Stack direction="row" justifyContent="flex-end">
        <Stack className={styles.buttons} spacing={1} direction="row-reverse">
          <Button onClick={handleClickSubmit} loading={loading} disabled={empty || isSlugDuplicate}>
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
