'use client'

import type { FunctionComponent } from 'react'
import Dialog from '@mui/material/Dialog'

import type { ExternalServiceDeleteDialogBodyProps } from '@/components/ExternalServiceDeleteDialogBody'
import ExternalServiceDeleteDialogBody from '@/components/ExternalServiceDeleteDialogBody'

const ExternalServiceDeleteDialog: FunctionComponent<ExternalServiceDeleteDialogProps> = ({
  close,
  ...props
}) => (
  <Dialog open onClose={close}>
    <ExternalServiceDeleteDialogBody close={close} {...props} />
  </Dialog>
)

export type ExternalServiceDeleteDialogProps = ExternalServiceDeleteDialogBodyProps

export default ExternalServiceDeleteDialog
