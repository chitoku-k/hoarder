'use client'

import type { ChangeEvent, FunctionComponent } from 'react'
import { useCallback, useEffect, useRef, useState } from 'react'
import Button from '@mui/material/Button'
import Chip from '@mui/material/Chip'
import LoadingButton from '@mui/lab/LoadingButton'
import Portal from '@mui/material/Portal'
import Snackbar from '@mui/material/Snackbar'
import Stack from '@mui/material/Stack'
import TextField from '@mui/material/TextField'

import { EXTERNAL_SERVICE_URL_PATTERN_INVALID, useError, useUpdateExternalService } from '@/hooks'
import type { ExternalService } from '@/types'

import styles from './styles.module.scss'

const ExternalServiceListColumnBodyEdit: FunctionComponent<ExternalServiceListColumnBodyEditProps> = ({
  externalService: current,
  close,
  onEdit,
}) => {
  const [ updateExternalService, { error, loading } ] = useUpdateExternalService()
  const { graphQLError } = useError()

  const ref = useCallback((input: HTMLElement) => {
    input?.focus({
      preventScroll: true,
    })
  }, [])

  const urlPatternRef = useRef<HTMLInputElement>(null)
  const [ urlPatternSelection, setUrlPatternSelection ] = useState<[number, number] | [null, null]>([null, null])

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

  const handleChangeUrlPattern = useCallback((e: ChangeEvent<HTMLInputElement>) => {
    const urlPattern = e.currentTarget.value
    setExternalService(externalService => ({
      ...externalService,
      urlPattern,
    }))
    setUrlPatternSelection([null, null])
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
    setUrlPatternSelection([selectionStart + prefix.length, selectionEnd + prefix.length])
  }, [ urlPatternRef ])

  useEffect(() => {
    const { current } = urlPatternRef
    if (!current) {
      return
    }

    const [ selectionStart, selectionEnd ] = urlPatternSelection
    if (selectionStart === null || selectionEnd === null) {
      return
    }

    current.setSelectionRange(selectionStart, selectionEnd)
    current.focus()
  }, [ urlPatternRef, urlPatternSelection ])

  const handleClickCancel = useCallback(() => {
    close()
  }, [ close ])

  const handleClickSubmit = useCallback(() => {
    updateExternalService(externalService).then(
      newExternalService => {
        close()
        onEdit(newExternalService)
      },
      e => {
        console.error('Error updating external service\n', e)
      },
    )
  }, [ externalService, updateExternalService, onEdit, close ])

  const externalServiceUrlPatternInvalid = graphQLError(error, EXTERNAL_SERVICE_URL_PATTERN_INVALID)
  const externalServiceUrlPatternInvalidDescription = externalServiceUrlPatternInvalid?.extensions.details.data.description
  const isUrlPatternInvalid = externalServiceUrlPatternInvalid?.extensions.details.data.urlPattern === externalService.urlPattern

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
          slotProps={{
            htmlInput: {
              readOnly: true,
              tabIndex: -1,
            },
          }}
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
                  {['id', 'creatorId'].map(name => (
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
