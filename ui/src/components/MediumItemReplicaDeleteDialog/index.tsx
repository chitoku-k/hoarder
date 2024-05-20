'use client'

import type { FunctionComponent } from 'react'
import Dialog from '@mui/material/Dialog'

import type { MediumItemReplicaDeleteDialogBodyProps } from '@/components/MediumItemReplicaDeleteDialogBody'
import MediumItemReplicaDeleteDialogBody from '@/components/MediumItemReplicaDeleteDialogBody'

const MediumItemReplicaDeleteDialog: FunctionComponent<MediumItemReplicaDeleteDialogProps> = ({
  close,
  ...props
}) => (
  <Dialog open onClose={close}>
    <MediumItemReplicaDeleteDialogBody close={close} {...props} />
  </Dialog>
)

export type MediumItemReplicaDeleteDialogProps = MediumItemReplicaDeleteDialogBodyProps

export default MediumItemReplicaDeleteDialog
