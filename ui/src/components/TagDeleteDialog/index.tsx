'use client'

import type { FunctionComponent } from 'react'
import { Suspense } from 'react'
import { ErrorBoundary } from 'react-error-boundary'
import Dialog from '@mui/material/Dialog'

import type { TagDeleteDialogBodyProps } from '@/components/TagDeleteDialogBody'
import TagDeleteDialogBody from '@/components/TagDeleteDialogBody'
import TagDeleteDialogError from '@/components/TagDeleteDialogError'
import TagDeleteDialogLoading from '@/components/TagDeleteDialogLoading'

const TagDeleteDialog: FunctionComponent<TagDeleteDialogProps> = ({
  close,
  ...props
}) => (
  <Dialog open onClose={close}>
    <ErrorBoundary fallback={<TagDeleteDialogError close={close} />}>
      <Suspense fallback={<TagDeleteDialogLoading />}>
        <TagDeleteDialogBody close={close} {...props} />
      </Suspense>
    </ErrorBoundary>
  </Dialog>
)

export type TagDeleteDialogProps = TagDeleteDialogBodyProps

export default TagDeleteDialog
