'use client'

import type { FunctionComponent } from 'react'
import { useCallback, useState } from 'react'
import InputAdornment from '@mui/material/InputAdornment'
import LoadingButton from '@mui/lab/LoadingButton'
import Stack from '@mui/material/Stack'
import { DateTimeField } from '@mui/x-date-pickers/DateTimeField'
import CalendarMonthIcon from '@mui/icons-material/CalendarMonth'

import type { MediumCreate } from '@/components/MediumCreateView'
import MediumItemMetadataHeader from '@/components/MediumItemMetadataHeader'

import styles from './styles.module.scss'

const MediumItemMetadataSummaryCreate: FunctionComponent<MediumItemMetadataSummaryCreateProps> = ({
  loading,
  save,
}) => {
  const [ medium, setMedium ] = useState<MediumCreate>({
    createdAt: null,
  })

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

  const handleClickSubmit = useCallback(() => {
    save(medium)
  }, [ save, medium ])

  return (
    <Stack>
      <MediumItemMetadataHeader title="メディア">
        <LoadingButton onClick={handleClickSubmit} loading={loading}>
          <span>保存</span>
        </LoadingButton>
      </MediumItemMetadataHeader>
      <Stack flexGrow={1} spacing={3}>
        <DateTimeField
          fullWidth
          variant="standard"
          disabled={loading}
          value={medium.createdAt ? new Date(medium.createdAt) : null}
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
      </Stack>
    </Stack>
  )
}

export interface MediumItemMetadataSummaryCreateProps {
  loading: boolean
  save: (medium: MediumCreate) => void
}

export default MediumItemMetadataSummaryCreate
