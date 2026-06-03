import type { FunctionComponent, ReactNode } from 'react'
import type { Metadata } from 'next'
import Grid from '@mui/material/Grid'
import Stack from '@mui/material/Stack'

import Header from '@/components/Header'
import ApolloWrapper from '@/components/ApolloWrapper'
import ThemeRegistry from '@/components/ThemeRegistry'
import LocalizationWrapper from '@/components/LocalizationWrapper'
import { SearchProvider } from '@/contexts'

import './global.scss'
import styles from './styles.module.scss'

export const dynamic = 'force-dynamic'

export const metadata = {
  title: {
    template: 'Hoarder: %s',
    default: 'Hoarder',
  },
} satisfies Metadata

const RootLayout: FunctionComponent<RootLayoutProps> = ({
  children,
}) => (
  <html lang="ja">
    <body>
      <LocalizationWrapper>
        <ApolloWrapper>
          <ThemeRegistry>
            <SearchProvider>
              <Stack className={styles.root}>
                <Grid size={12}>
                  <Header />
                </Grid>
                <Grid className={styles.body} size={12}>
                  {children}
                </Grid>
              </Stack>
            </SearchProvider>
          </ThemeRegistry>
        </ApolloWrapper>
      </LocalizationWrapper>
    </body>
  </html>
)

export interface RootLayoutProps {
  readonly children: ReactNode
}

export default RootLayout
