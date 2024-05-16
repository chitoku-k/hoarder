'use client'

import type { FunctionComponent } from 'react'
import { useCallback, useState } from 'react'
import Button from '@mui/material/Button'
import InputAdornment from '@mui/material/InputAdornment'
import LoadingButton from '@mui/lab/LoadingButton'
import Stack from '@mui/material/Stack'
import { DateTimeField } from '@mui/x-date-pickers/DateTimeField'
import CalendarMonthIcon from '@mui/icons-material/CalendarMonth'

import MediumDeleteDialog from '@/components/MediumDeleteDialog'
import MediumItemMetadataHeader from '@/components/MediumItemMetadataHeader'
import type { Medium } from '@/types'

import styles from './styles.module.scss'

const MediumItemMetadataSummaryEdit: FunctionComponent<MediumItemMetadataSummaryEditProps> = ({
  loading,
  medium: current,
  save,
  close,
  onDelete,
}) => {
  const [ medium, setMedium ] = useState(current)
  const [ deleting, setDeleting ] = useState(false)

  const handleChangeCreatedAt = useCallback((value: Date | null) => {
    if (!value || isNaN(value.getTime())) {
      return
    }

    const createdAt = value.toISOString()
    setMedium(medium => ({
      ...medium,
      createdAt,
    }))
  }, [])

  const handleClickCancel = useCallback(() => {
    close?.()
  }, [ close ])

  const handleClickSubmit = useCallback(() => {
    save(medium)
  }, [ save, medium ])

  const handleClickDeleteMedium = useCallback(() => {
    setDeleting(true)
  }, [])

  const closeDeleteMedium = useCallback(() => {
    setDeleting(false)
  }, [])

  const handleDeleteMedium = useCallback(() => {
    onDelete()
  }, [ onDelete ])

  return (
    <Stack>
      <MediumItemMetadataHeader title="メディア">
        <LoadingButton onClick={handleClickSubmit} loading={loading}>
          <span>保存</span>
        </LoadingButton>
        <Button onClick={handleClickCancel}>
          キャンセル
        </Button>
      </MediumItemMetadataHeader>
      <Stack flexGrow={1} spacing={3}>
        <DateTimeField
          fullWidth
          variant="standard"
          disabled={loading}
          value={new Date(medium.createdAt)}
          format="Ppp"
          onChange={handleChangeCreatedAt}
          shouldRespectLeadingZeros
          slotProps={{
            textField: {
              InputProps: {
                startAdornment: (
                  <InputAdornment position="start">
                    <CalendarMonthIcon className={styles.icon} fontSize="small" />
                  </InputAdornment>
                ),
                endAdornment: (
                  <InputAdornment position="end">
                    登録
                  </InputAdornment>
                ),
              },
            },
          }}
        />
        <Button variant="outlined" color="error" onClick={handleClickDeleteMedium}>削除</Button>
      </Stack>
      {deleting ? (
        <MediumDeleteDialog key={current.id} medium={current} close={closeDeleteMedium} onDelete={handleDeleteMedium} />
      ) : null}
    </Stack>
  )
}

export interface MediumItemMetadataSummaryEditProps {
  loading: boolean
  medium: Medium
  save: (medium: Medium) => void
  close?: () => void
  onDelete: () => void
}

export default MediumItemMetadataSummaryEdit
