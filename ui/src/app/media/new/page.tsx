import type { FunctionComponent } from 'react'
import type { Metadata } from 'next'

import Content from '@/components/Content'
import MediumCreate from '@/components/MediumCreate'

export const metadata: Metadata = {
  title: 'メディアの追加',
}

const Page: FunctionComponent = () => (
  <Content xs={12}>
    <MediumCreate />
  </Content>
)

export default Page
