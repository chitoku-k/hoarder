'use client'

import type { FunctionComponent } from 'react'
import clsx from 'clsx'
import { skipToken } from '@apollo/client/react'
import Breadcrumbs from '@mui/material/Breadcrumbs'
import Stack from '@mui/material/Stack'
import Typography from '@mui/material/Typography'
import NavigateNextIcon from '@mui/icons-material/NavigateNext'
import SellIcon from '@mui/icons-material/Sell'

import { ancestors } from '@/components/TagListView'
import { useTag } from '@/hooks'
import type { Tag } from '@/types'

import styles from './styles.module.scss'

const TagBreadcrumbsListIcon: FunctionComponent = () => (
  <NavigateNextIcon className={styles.icon} fontSize="small" />
)

const useTagByProps = (props: TagBreadcrumbsListBodyProps): Tag | null => {
  let result: Tag | null = null
  let id: string | null = null

  if ('tag' in props) {
    if (props.tag.parent !== void 0) {
      result = props.tag
    } else {
      id = props.tag.id
    }
  } else if ('id' in props) {
    id = props.id
  }

  const tag = useTag(id === null ? skipToken : { id })
  return result ?? tag
}

const TagBreadcrumbsListBody: FunctionComponent<TagBreadcrumbsListBodyProps> = ({
  className,
  root,
  parent,
  noWrap,
  ...props
}) => {
  const tag = useTagByProps(props)

  const current = parent
    ? tag?.parent ?? null
    : tag

  const hierarchy = current
    ? [ ...ancestors(current) ]
    : []

  return (
    <Stack
      className={clsx(styles.breadcrumbsContainer, noWrap && styles.nowrap, className)}
      direction="row"
      spacing={1}
      alignItems="start"
    >
      <SellIcon className={styles.icon} fontSize="small" />
      <Breadcrumbs className={clsx(styles.breadcrumbs, noWrap && styles.nowrap)} separator={null}>
        {hierarchy.map(({ id, name }) => (
          <Stack key={id} direction="row">
            <TagBreadcrumbsListIcon />
            <Typography className={styles.name} noWrap={noWrap}>
              {name}
            </Typography>
          </Stack>
        ))}
        {root ? (
          <Stack>
            <TagBreadcrumbsListIcon />
          </Stack>
        ) : null}
      </Breadcrumbs>
    </Stack>
  )
}

interface TagBreadcrumbsListBodyPropsBase {
  className?: string
  root?: boolean
  parent?: boolean
  noWrap?: boolean
}

interface TagBreadcrumbsListBodyPropsByTag extends TagBreadcrumbsListBodyPropsBase {
  tag: Tag
}

interface TagBreadcrumbsListBodyPropsByTagID extends TagBreadcrumbsListBodyPropsBase {
  id: string
}

export type TagBreadcrumbsListBodyProps = TagBreadcrumbsListBodyPropsBase | TagBreadcrumbsListBodyPropsByTag | TagBreadcrumbsListBodyPropsByTagID

export default TagBreadcrumbsListBody
