'use client'

import type { FunctionComponent, MouseEventHandler, ReactNode } from 'react'
import clsx from 'clsx'
import ListItemButton from '@mui/material/ListItemButton'
import ListItemIcon from '@mui/material/ListItemIcon'
import ListItemText from '@mui/material/ListItemText'
import FolderSpecialIcon from '@mui/icons-material/FolderSpecial'

import styles from './styles.module.scss'

const ExternalServiceListColumnBodyListItem: FunctionComponent<ExternalServiceListColumnBodyListItemProps> = ({
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
    className={clsx(className, styles.externalService)}
    disabled={disabled}
    selected={selected}
    onClick={disabled ? undefined : onClick}
  >
    <ListItemIcon className={clsx(styles.icon, dense && styles.iconDense)}>
      <FolderSpecialIcon fontSize={dense ? 'small' : 'medium'} />
    </ListItemIcon>
    <ListItemText
      className={styles.text}
      primary={primary}
      primaryTypographyProps={{
        noWrap: true,
      }}
      secondary={secondary}
      secondaryTypographyProps={{
        noWrap: true,
      }}
    />
    {children}
  </ListItemButton>
)

export interface ExternalServiceListColumnBodyListItemProps {
  className?: string
  dense?: boolean
  disabled?: boolean
  selected?: boolean
  primary?: ReactNode
  secondary?: ReactNode
  children?: ReactNode
  onClick?: MouseEventHandler<HTMLElement>
}

export default ExternalServiceListColumnBodyListItem
