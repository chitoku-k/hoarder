import type { FunctionComponent } from 'react'
import Dialog from '@mui/material/Dialog'

import type { TagTypeDeleteDialogBodyProps } from '@/components/TagTypeDeleteDialogBody'
import TagTypeDeleteDialogBody from '@/components/TagTypeDeleteDialogBody'

const TagTypeDeleteDialog: FunctionComponent<TagTypeDeleteDialogProps> = ({
  close,
  ...props
}) => (
  <Dialog open onClose={close}>
    <TagTypeDeleteDialogBody close={close} {...props} />
  </Dialog>
)

export type TagTypeDeleteDialogProps = TagTypeDeleteDialogBodyProps

export default TagTypeDeleteDialog
