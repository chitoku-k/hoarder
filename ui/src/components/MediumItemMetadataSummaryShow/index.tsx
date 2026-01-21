import type { FunctionComponent } from 'react'
import Button from '@mui/material/Button'
import Stack from '@mui/material/Stack'
import Typography from '@mui/material/Typography'
import CalendarMonthIcon from '@mui/icons-material/CalendarMonth'

import DateTime from '@/components/DateTime'
import MediumItemMetadataHeader from '@/components/MediumItemMetadataHeader'
import type { Medium } from '@/types'

import styles from './styles.module.scss'

const MediumItemMetadataSummaryShow: FunctionComponent<MediumItemMetadataSummaryShowProps> = ({
  medium,
  edit,
}) => (
  <Stack>
    <MediumItemMetadataHeader title="メディア">
      <Button onClick={edit}>編集</Button>
    </MediumItemMetadataHeader>
    <Stack className={styles.createdAt} direction="row" spacing={1} alignItems="center">
      <CalendarMonthIcon className={styles.icon} fontSize="small" />
      <Stack direction="row" spacing={0.5}>
        <DateTime date={new Date(medium.createdAt)} format="Pp" />
        <Typography>登録</Typography>
      </Stack>
    </Stack>
  </Stack>
)

export interface MediumItemMetadataSummaryShowProps {
  readonly medium: Medium
  readonly edit: () => void
}

export default MediumItemMetadataSummaryShow
