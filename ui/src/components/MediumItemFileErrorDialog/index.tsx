'use client'

import type { FunctionComponent } from 'react'
import Dialog from '@mui/material/Dialog'

import type { MediumItemFileErrorDialogBodyProps } from '@/components/MediumItemFileErrorDialogBody'
import MediumItemFileErrorDialogBody from '@/components/MediumItemFileErrorDialogBody'

const MediumItemFileErrorDialog: FunctionComponent<MediumItemFileErrorDialogProps> = ({
  close,
  ...props
}) => (
  <Dialog open onClose={close}>
    <MediumItemFileErrorDialogBody close={close} {...props} />
  </Dialog>
)

export type MediumItemFileErrorDialogProps = MediumItemFileErrorDialogBodyProps

export default MediumItemFileErrorDialog
