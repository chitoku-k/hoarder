import type { FunctionComponent, ReactNode } from 'react'
import type { Grid2Props } from '@mui/material/Grid2'
import Grid from '@mui/material/Grid2'
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

export interface ContentProps extends Grid2Props {
  children: ReactNode
}

export default Content
