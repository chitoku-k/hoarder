import type { FunctionComponent } from 'react'
import Dialog from '@mui/material/Dialog'

import type { MediumItemFileOverwriteDialogBodyProps } from '@/components/MediumItemFileOverwriteDialogBody'
import MediumItemFileOverwriteDialogBody from '@/components/MediumItemFileOverwriteDialogBody'

import styles from './styles.module.scss'

const MediumItemFileOverwriteDialog: FunctionComponent<MediumItemFileOverwriteDialogProps> = ({
  close,
  ...props
}) => (
  <Dialog className={styles.dialog} open onClose={close}>
    <MediumItemFileOverwriteDialogBody close={close} {...props} />
  </Dialog>
)

export type MediumItemFileOverwriteDialogProps = MediumItemFileOverwriteDialogBodyProps

export default MediumItemFileOverwriteDialog
