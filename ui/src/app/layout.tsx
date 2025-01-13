import type { FunctionComponent, ReactNode } from 'react'
import type { Metadata } from 'next'
import Grid from '@mui/material/Grid2'

import Header from '@/components/Header'
import ApolloWrapper from '@/components/ApolloWrapper'
import ThemeRegistry from '@/components/ThemeRegistry'
import LocalizationWrapper from '@/components/LocalizationWrapper'
import { SearchProvider } from '@/contexts'

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
            <SearchProvider>
              <Grid container direction="column" height="100%" flexWrap="nowrap">
                <Grid size={12}>
                  <Header />
                </Grid>
                <Grid size={12} flexGrow={1}>
                  {children}
                </Grid>
              </Grid>
            </SearchProvider>
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
