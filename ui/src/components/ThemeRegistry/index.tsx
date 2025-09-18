'use client'

import type { FunctionComponent, ReactNode } from 'react'
import { CssBaseline, ThemeProvider } from '@mui/material'
import { AppRouterCacheProvider } from '@mui/material-nextjs/v14-appRouter'

import '@fontsource/roboto/300.css'
import '@fontsource/roboto/400.css'
import '@fontsource/roboto/500.css'
import '@fontsource/roboto/700.css'

import theme from './theme'

const ThemeRegistry: FunctionComponent<ThemeRegistryProps> = ({
  children,
}) => (
  <AppRouterCacheProvider options={{ enableCssLayer: true }}>
    <ThemeProvider theme={theme}>
      <CssBaseline />
      {children}
    </ThemeProvider>
  </AppRouterCacheProvider>
)

export interface ThemeRegistryProps {
  readonly children: ReactNode
}

export default ThemeRegistry
