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
  </ListItemButton>
)

export interface ExternalServiceListColumnBodyListItemProps {
  readonly className?: string
  readonly dense?: boolean
  readonly disabled?: boolean
  readonly selected?: boolean
  readonly primary?: ReactNode
  readonly secondary?: ReactNode
  readonly children?: ReactNode
  readonly onClick?: MouseEventHandler<HTMLElement>
}

export default ExternalServiceListColumnBodyListItem
