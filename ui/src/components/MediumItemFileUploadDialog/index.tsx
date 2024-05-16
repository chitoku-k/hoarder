'use client'

import type { FunctionComponent } from 'react'
import Dialog from '@mui/material/Dialog'

import type { MediumItemFileUploadDialogBodyProps } from '@/components/MediumItemFileUploadDialogBody'
import MediumItemFileUploadDialogBody from '@/components/MediumItemFileUploadDialogBody'

import styles from './styles.module.scss'

const MediumItemFileUploadDialog: FunctionComponent<MediumItemFileUploadDialogProps> = ({
  close,
  ...props
}) => (
  <Dialog className={styles.dialog} open onClose={close}>
    <MediumItemFileUploadDialogBody close={close} {...props} />
  </Dialog>
)

export type MediumItemFileUploadDialogProps = MediumItemFileUploadDialogBodyProps

export default MediumItemFileUploadDialog
