import type { FunctionComponent, ReactNode } from 'react'
import type { Metadata } from 'next'
import Grid from '@mui/material/Unstable_Grid2'

import Header from '@/components/Header'
import ApolloWrapper from '@/components/ApolloWrapper'
import ThemeRegistry from '@/components/ThemeRegistry'
import LocalizationWrapper from '@/components/LocalizationWrapper'

import './global.scss'

export const dynamic = 'force-dynamic'

export const metadata: Metadata = {
  title: {
    template: 'Hoarder: %s',
    default: 'Hoarder',
  },
}

const RootLayout: FunctionComponent<RootLayoutProps> = ({
  children,
}) => (
  <html lang="ja">
    <body>
      <LocalizationWrapper>
        <ApolloWrapper>
          <ThemeRegistry>
            <Grid container>
              <Grid xs={12}>
                <Header />
              </Grid>
              <Grid xs={12}>
                {children}
              </Grid>
            </Grid>
          </ThemeRegistry>
        </ApolloWrapper>
      </LocalizationWrapper>
    </body>
  </html>
)

export interface RootLayoutProps {
  children: ReactNode
}

export default RootLayout
