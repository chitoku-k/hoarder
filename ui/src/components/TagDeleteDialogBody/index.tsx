'use client'

import type { ChangeEvent, FunctionComponent } from 'react'
import { useCallback, useState } from 'react'
import { skipToken } from '@apollo/client/react'
import Button from '@mui/material/Button'
import Checkbox from '@mui/material/Checkbox'
import DialogActions from '@mui/material/DialogActions'
import DialogContent from '@mui/material/DialogContent'
import DialogContentText from '@mui/material/DialogContentText'
import FormControlLabel from '@mui/material/FormControlLabel'
import FormGroup from '@mui/material/FormGroup'
import Typography from '@mui/material/Typography'

import { TAG_CHILDREN_EXIST, useDeleteTag, useError, useTag } from '@/hooks'
import type { Tag } from '@/types'

import styles from './styles.module.scss'

const useTagByProps = (props: Pick<TagDeleteDialogBodyProps, 'tag'>): Tag | null => {
  let result: Tag | null = null
  let id: string | null = null

  if (props.tag.children) {
    result = props.tag
  } else {
    id = props.tag.id
  }

  const tag = useTag(id === null ? skipToken : { id })
  return result ?? tag
}

const TagDeleteDialogBody: FunctionComponent<TagDeleteDialogBodyProps> = ({
  close,
  onDelete,
  ...props
}) => {
  const [ deleteTag, { error, loading } ] = useDeleteTag()
  const { tag } = props
  const children = useTagByProps(props)?.children

  const { graphQLError } = useError()
  const [ recursive, setRecursive ] = useState(false)

  const handleChangeRecursive = useCallback((e: ChangeEvent<HTMLInputElement>) => {
    setRecursive(e.currentTarget.checked)
  }, [])

  const handleClickDelete = useCallback(async () => {
    try {
      await deleteTag({
        id: tag.id,
        recursive,
      })
      close()
      onDelete(tag)
    } catch (e) {
      console.error('Error deleting tag\n', e)
    }
  }, [ deleteTag, tag, recursive, onDelete, close ])

  const tagChildrenExist = graphQLError(error, TAG_CHILDREN_EXIST)
  const hasChildren = Boolean(children?.length ?? tagChildrenExist)

  return error && !tagChildrenExist ? (
    <>
      <DialogContent>
        <DialogContentText>タグを削除できませんでした</DialogContentText>
      </DialogContent>
      <DialogActions>
        <Button onClick={close} autoFocus>閉じる</Button>
      </DialogActions>
    </>
  ) : (
    <>
      <DialogContent>
        <DialogContentText>
          {hasChildren ? (
            <>
              タグ「
              <Typography component="strong" fontWeight="bold">{tag.name}</Typography>
              」には子タグがあります。削除しますか？
            </>
          ) : (
            <>
              タグ「
              <Typography component="strong" fontWeight="bold">{tag.name}</Typography>
              」を削除しますか？
            </>
          )}
        </DialogContentText>
        {hasChildren ? (
          <FormGroup>
            <FormControlLabel
              className={styles.label}
              control={<Checkbox checked={recursive} onChange={handleChangeRecursive} />}
              label="子タグをすべて削除する"
            />
          </FormGroup>
        ) : null}
      </DialogContent>
      <DialogActions>
        <Button onClick={close} autoFocus>キャンセル</Button>
        <Button color="error" onClick={handleClickDelete} loading={loading} disabled={hasChildren && !recursive}>削除</Button>
      </DialogActions>
    </>
  )
}

export interface TagDeleteDialogBodyProps {
  readonly tag: Tag
  readonly close: () => void
  readonly onDelete: (tag: Tag) => void
}

export default TagDeleteDialogBody
