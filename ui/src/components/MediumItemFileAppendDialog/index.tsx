import type { FunctionComponent } from 'react'
import { Suspense } from 'react'
import Dialog from '@mui/material/Dialog'

import type { MediumItemFileAppendDialogBodyProps } from '@/components/MediumItemFileAppendDialogBody'
import MediumItemFileAppendDialogBody from '@/components/MediumItemFileAppendDialogBody'
import type { MediumItemFileAppendDialogLoadingProps } from '@/components/MediumItemFileAppendDialogLoading'
import MediumItemFileAppendDialogLoading from '@/components/MediumItemFileAppendDialogLoading'

const MediumItemFileAppendDialog: FunctionComponent<MediumItemFileAppendDialogProps> = ({
  close,
  cancel,
  ...props
}) => (
  <Dialog open onClose={close}>
    <Suspense fallback={<MediumItemFileAppendDialogLoading cancel={cancel} />}>
      <MediumItemFileAppendDialogBody close={close} {...props} />
    </Suspense>
  </Dialog>
)

export type MediumItemFileAppendDialogProps = MediumItemFileAppendDialogBodyProps & MediumItemFileAppendDialogLoadingProps

export default MediumItemFileAppendDialog
