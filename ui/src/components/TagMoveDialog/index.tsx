import type { FunctionComponent } from 'react'
import { Suspense } from 'react'
import { ErrorBoundary } from 'react-error-boundary'
import Dialog from '@mui/material/Dialog'

import type { TagMoveDialogBodyProps } from '@/components/TagMoveDialogBody'
import TagMoveDialogBody from '@/components/TagMoveDialogBody'
import TagMoveDialogError from '@/components/TagMoveDialogError'
import TagMoveDialogLoading from '@/components/TagMoveDialogLoading'

import styles from './styles.module.scss'

const TagMoveDialog: FunctionComponent<TagMoveDialogProps> = ({
  close,
  ...props
}) => (
  <Dialog className={styles.dialog} open onClose={close}>
    <ErrorBoundary fallback={<TagMoveDialogError close={close} />}>
      <Suspense fallback={<TagMoveDialogLoading />}>
        <TagMoveDialogBody close={close} {...props} />
      </Suspense>
    </ErrorBoundary>
  </Dialog>
)

export type TagMoveDialogProps = TagMoveDialogBodyProps

export default TagMoveDialog
