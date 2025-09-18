import type { FunctionComponent } from 'react'
import type { Metadata } from 'next'

import Content from '@/components/Content'
import MediumCreate from '@/components/MediumCreate'

export const metadata = {
  title: 'メディアの追加',
} satisfies Metadata

const Page: FunctionComponent = () => (
  <Content size={12}>
    <MediumCreate />
  </Content>
)

export default Page
