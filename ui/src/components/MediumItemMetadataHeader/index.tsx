import type { FunctionComponent, ReactNode } from 'react'
import Stack from '@mui/material/Stack'
import Typography from '@mui/material/Typography'

import styles from './styles.module.scss'

const MediumItemMetadataHeader: FunctionComponent<MediumItemMetadataHeaderProps> = ({
  title,
  children,
}) => (
  <Stack className={styles.header} direction="row" alignItems="center" justifyContent="space-between">
    <Typography className={styles.title} variant="h3">{title}</Typography>
    <Stack spacing={1} direction="row-reverse">
      {children}
    </Stack>
  </Stack>
)

export interface MediumItemMetadataHeaderProps {
  readonly title: string
  readonly children?: ReactNode
}

export default MediumItemMetadataHeader
