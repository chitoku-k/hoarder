'use client'

import type { FunctionComponent } from 'react'
import { useCallback, useState } from 'react'
import Button from '@mui/material/Button'
import InputAdornment from '@mui/material/InputAdornment'
import Stack from '@mui/material/Stack'
import { DateTimeField } from '@mui/x-date-pickers/DateTimeField'
import CalendarMonthIcon from '@mui/icons-material/CalendarMonth'

import type { MediumCreate } from '@/components/MediumCreateView'
import MediumItemMetadataHeader from '@/components/MediumItemMetadataHeader'
import { useBeforeUnload } from '@/hooks'

import styles from './styles.module.scss'

const hasChanges = (medium: MediumCreate) => Boolean(medium.createdAt)

const MediumItemMetadataSummaryCreate: FunctionComponent<MediumItemMetadataSummaryCreateProps> = ({
  loading,
  save,
}) => {
  const [ medium, setMedium ] = useState<MediumCreate>({
    createdAt: null,
  })

  const handleChangeCreatedAt = useCallback((value: Date | null) => {
    const createdAt = value && !isNaN(value.getTime()) ? value.toISOString() : null
    setMedium(medium => ({
      ...medium,
      createdAt,
    }))
  }, [])

  const handleClickSubmit = useCallback(() => {
    save(medium)
  }, [ save, medium ])

  const changed = hasChanges(medium)
  useBeforeUnload(changed)

  return (
    <Stack>
      <MediumItemMetadataHeader title="メディア">
        <Button onClick={handleClickSubmit} loading={loading}>
          保存
        </Button>
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
