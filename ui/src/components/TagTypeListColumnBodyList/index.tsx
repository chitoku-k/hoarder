'use client'

import type { FunctionComponent, MouseEvent } from 'react'
import { useCallback, useState } from 'react'
import Button from '@mui/material/Button'
import IconButton from '@mui/material/IconButton'
import List from '@mui/material/List'
import Stack from '@mui/material/Stack'
import DeleteOutlinedIcon from '@mui/icons-material/DeleteOutlined'
import EditOutlinedIcon from '@mui/icons-material/EditOutlined'

import TagTypeListColumnBodyListItem from '@/components/TagTypeListColumnBodyListItem'
import { useAllTagTypes } from '@/hooks'
import type { TagType } from '@/types'

import styles from './styles.module.scss'

const TagTypeListColumnBodyList: FunctionComponent<TagTypeListColumnBodyListProps> = ({
  creating,
  editing,
  active,
  readonly,
  dense,
  disabled: disabledTagType,
  onSelect: onSelectTagType,
  show: showTagType,
  create: createTagType,
  edit: editTagType,
  delete: deleteTagType,
}) => {
  const allTagTypes = useAllTagTypes()

  const [ scrollTop, setScrollTop ] = useState(0)
  const ref = useCallback((node: HTMLElement | null) => {
    if (!node) {
      return
    }
    if (creating) {
      setScrollTop(node.scrollTop)
      node.scrollTo({
        top: node.scrollHeight,
        behavior: 'smooth',
      })
    } else {
      node.scrollTo({
        top: scrollTop,
        behavior: 'smooth',
      })
    }
  }, [ creating, scrollTop ])

  const handleClickTagType = (tagType: TagType) => {
    onSelectTagType?.(tagType)
    showTagType(tagType)
  }

  const handleClickCreateTagType = useCallback(() => {
    createTagType()
  }, [ createTagType ])

  const handleClickEditTagType = (e: MouseEvent<HTMLButtonElement>, tagType: TagType) => {
    editTagType(tagType)
    e.stopPropagation()
  }

  const handleClickDeleteTagType = (e: MouseEvent<HTMLButtonElement>, tagType: TagType) => {
    deleteTagType(tagType)
    e.stopPropagation()
  }

  const handleMouseDownEditTagType = useCallback((e: MouseEvent<HTMLButtonElement>) => {
    e.stopPropagation()
  }, [])

  const handleMouseDownDeleteTagType = useCallback((e: MouseEvent<HTMLButtonElement>) => {
    e.stopPropagation()
  }, [])

  return (
    <Stack className={styles.container}>
      <Stack className={styles.buttons}>
        {!readonly ? (
          <Button variant="outlined" onClick={handleClickCreateTagType}>
            新規作成
          </Button>
        ) : null}
      </Stack>
      <List ref={ref} dense={dense} className={styles.tagTypes}>
        {allTagTypes.map(tagType => (
          <TagTypeListColumnBodyListItem
            key={tagType.id}
            className={styles.tagType}
            dense={dense}
            disabled={Boolean(disabledTagType?.(tagType))}
            selected={!creating && (editing ?? active)?.id === tagType.id}
            primary={tagType.name}
            onClick={() => handleClickTagType(tagType)}
          >
            {!readonly ? (
              <>
                <IconButton
                  className={styles.tagTypeButton}
                  size="small"
                  onMouseDown={handleMouseDownEditTagType}
                  onClick={e => handleClickEditTagType(e, tagType)}
                >
                  <EditOutlinedIcon fontSize={dense ? 'small' : 'medium'} />
                </IconButton>
                <IconButton
                  className={styles.tagTypeButton}
                  size="small"
                  onMouseDown={handleMouseDownDeleteTagType}
                  onClick={e => handleClickDeleteTagType(e, tagType)}
                >
                  <DeleteOutlinedIcon fontSize={dense ? 'small' : 'medium'} />
                </IconButton>
              </>
            ) : null}
          </TagTypeListColumnBodyListItem>
        ))}
        {creating ? (
          <TagTypeListColumnBodyListItem
            className={styles.tagType}
            dense={dense}
            selected
            primary="新しいタイプ"
          />
        ) : null}
      </List>
    </Stack>
  )
}

export interface TagTypeColumn {
  creating: boolean
  editing: TagType | null
  active: TagType | null
}

export interface TagTypeListColumnBodyListProps extends TagTypeColumn {
  readonly: boolean
  dense: boolean
  disabled?: (tagType: TagType) => boolean
  onSelect?: (tagType: TagType) => void
  create: () => void
  show: (tagType: TagType) => void
  edit: (tagType: TagType) => void
  delete: (tagType: TagType) => void
}

export default TagTypeListColumnBodyList
