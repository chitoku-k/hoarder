import type { FunctionComponent, ReactNode } from 'react'
import type { GridProps } from '@mui/material/Grid'
import Grid from '@mui/material/Grid'
import Toolbar from '@mui/material/Toolbar'

import styles from './styles.module.scss'

const Content: FunctionComponent<ContentProps> = ({
  children,
  ...props
}) => (
  <Grid className={styles.content} container direction="column" {...props}>
    <Toolbar />
    <Grid component="main" container direction="column" flexGrow={1}>
      {children}
    </Grid>
  </Grid>
)

export interface ContentProps extends GridProps {
  readonly children: ReactNode
}

export default Content
