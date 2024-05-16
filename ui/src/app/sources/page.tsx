import type { FunctionComponent } from 'react'
import type { Metadata } from 'next'

import Content from '@/components/Content'
import SourceList from '@/components/SourceList'

export const metadata: Metadata = {
  title: 'ソース',
}

const Page: FunctionComponent = () => (
  <Content>
    <SourceList />
  </Content>
)

export default Page
