'use client'

import type { FunctionComponent } from 'react'
import Dialog from '@mui/material/Dialog'

import type { MediumDeleteDialogBodyProps } from '@/components/MediumDeleteDialogBody'
import MediumDeleteDialogBody from '@/components/MediumDeleteDialogBody'

const MediumDeleteDialog: FunctionComponent<MediumDeleteDialogProps> = ({
  close,
  ...props
}) => (
  <Dialog open onClose={close}>
    <MediumDeleteDialogBody close={close} {...props} />
  </Dialog>
)

export type MediumDeleteDialogProps = MediumDeleteDialogBodyProps

export default MediumDeleteDialog
