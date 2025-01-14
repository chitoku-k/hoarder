'use client'

import type { FunctionComponent, MouseEventHandler, ReactNode } from 'react'
import clsx from 'clsx'
import ListItemButton from '@mui/material/ListItemButton'
import ListItemIcon from '@mui/material/ListItemIcon'
import ListItemText from '@mui/material/ListItemText'
import NavigateNextIcon from '@mui/icons-material/NavigateNext'
import SellIcon from '@mui/icons-material/Sell'

import styles from './styles.module.scss'

const TagListColumnBodyListItem: FunctionComponent<TagListColumnBodyListItemProps> = ({
  className,
  dense,
  disabled,
  selected,
  primary,
  secondary,
  children,
  onClick,
}) => (
  <ListItemButton
    className={clsx(className, styles.tag)}
    disabled={disabled}
    selected={selected}
    onClick={disabled ? undefined : onClick}
  >
    <ListItemIcon className={clsx(styles.icon, dense && styles.iconDense)}>
      <SellIcon fontSize={dense ? 'small' : 'medium'} />
    </ListItemIcon>
    <ListItemText
      className={styles.text}
      primary={primary}
      secondary={secondary}
      slotProps={{
        primary: {
          noWrap: true,
        },
        secondary: {
          noWrap: true,
        },
      }}
    />
    {children}
    <span className={styles.arrowContainer}>
      <NavigateNextIcon className={styles.arrow} />
    </span>
  </ListItemButton>
)

export interface TagListColumnBodyListItemProps {
  className?: string
  dense?: boolean
  disabled?: boolean
  selected?: boolean
  primary?: ReactNode
  secondary?: ReactNode
  children?: ReactNode
  onClick?: MouseEventHandler<HTMLElement>
}

export default TagListColumnBodyListItem
