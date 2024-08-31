import type { FunctionComponent, ReactNode } from 'react'
import type { Grid2Props } from '@mui/material/Grid2'
import Grid from '@mui/material/Grid2'
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

export interface ContentProps extends Grid2Props {
  children: ReactNode
}

export default Content
