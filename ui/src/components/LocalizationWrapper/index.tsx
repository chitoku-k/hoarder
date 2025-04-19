'use client'

import type { FunctionComponent, ReactNode } from 'react'
import { I18nProvider } from '@react-aria/i18n'
import { LocalizationProvider } from '@mui/x-date-pickers/LocalizationProvider'
import { AdapterDateFns } from '@mui/x-date-pickers/AdapterDateFns'
import { ja } from 'date-fns/locale/ja'

const LocalizationWrapper: FunctionComponent<LocalizationWrapperProps> = ({
  children,
}) => (
  <LocalizationProvider dateAdapter={AdapterDateFns} adapterLocale={ja}>
    <I18nProvider locale="ja">
      {children}
    </I18nProvider>
  </LocalizationProvider>
)

export interface LocalizationWrapperProps {
  children: ReactNode
}

export default LocalizationWrapper
