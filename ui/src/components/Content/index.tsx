import type { FunctionComponent, ReactNode } from 'react'
import type { StackProps } from '@mui/material/Stack'
import Stack from '@mui/material/Stack'
import Toolbar from '@mui/material/Toolbar'

import styles from './styles.module.scss'

const Content: FunctionComponent<ContentProps> = ({
  children,
  ...props
}) => (
  <Stack className={styles.content} {...props}>
    <Toolbar />
    <Stack className={styles.body} component="main">
      {children}
    </Stack>
  </Stack>
)

export interface ContentProps extends StackProps {
  readonly children: ReactNode
}

export default Content
