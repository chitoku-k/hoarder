'use client'

import type { FunctionComponent } from 'react'
import Dialog from '@mui/material/Dialog'

import type { MediumItemFileUploadAbortDialogBodyProps } from '@/components/MediumItemFileUploadAbortDialogBody'
import MediumItemFileUploadAbortDialogBody from '@/components/MediumItemFileUploadAbortDialogBody'

const MediumItemFileUploadAbortDialog: FunctionComponent<MediumItemFileUploadAbortDialogProps> = ({
  close,
  ...props
}) => (
  <Dialog open onClose={close}>
    <MediumItemFileUploadAbortDialogBody close={close} {...props} />
  </Dialog>
)

export type MediumItemFileUploadAbortDialogProps = MediumItemFileUploadAbortDialogBodyProps

export default MediumItemFileUploadAbortDialog
