'use client'

import type { ComponentType, FunctionComponent } from 'react'
import type { SvgIconProps } from '@mui/material'
import Filter2Icon from '@mui/icons-material/Filter2'
import Filter3Icon from '@mui/icons-material/Filter3'
import Filter4Icon from '@mui/icons-material/Filter4'
import Filter5Icon from '@mui/icons-material/Filter5'
import Filter6Icon from '@mui/icons-material/Filter6'
import Filter7Icon from '@mui/icons-material/Filter7'
import Filter8Icon from '@mui/icons-material/Filter8'
import Filter9Icon from '@mui/icons-material/Filter9'
import Filter9PlusIcon from '@mui/icons-material/Filter9Plus'

const icons: (ComponentType<SvgIconProps> | null)[] = [
  null,
  Filter2Icon,
  Filter3Icon,
  Filter4Icon,
  Filter5Icon,
  Filter6Icon,
  Filter7Icon,
  Filter8Icon,
  Filter9Icon,
  Filter9PlusIcon,
]

const MediumListItemCount: FunctionComponent<MediumListItemCountProps> = ({
  count,
  ...props
}) => {
  if (!count) {
    return null
  }

  const Icon = icons[Math.min(count, icons.length) - 1]
  return Icon ? (
    <Icon {...props} />
  ) : null
}

export interface MediumListItemCountProps extends SvgIconProps {
  count?: number | null
}

export default MediumListItemCount
