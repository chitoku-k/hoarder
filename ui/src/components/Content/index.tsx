import type { FunctionComponent, ReactNode } from 'react'
import type { GridDefaultBreakpoints } from '@mui/system'
import Grid from '@mui/material/Unstable_Grid2'
import Toolbar from '@mui/material/Toolbar'

import styles from './styles.module.scss'

const Content: FunctionComponent<ContentProps> = ({
  children,
  ...props
}) => (
  <Grid container>
    <Grid className={styles.main} {...props}>
      <Toolbar />
      <main>
        {children}
      </main>
    </Grid>
  </Grid>
)

export interface ContentProps extends GridDefaultBreakpoints {
  children: ReactNode
}

export default Content
