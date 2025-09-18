import type { FunctionComponent } from 'react'
import type { Metadata } from 'next'

import Content from '@/components/Content'
import SourceList from '@/components/SourceList'

export const metadata = {
  title: 'ソース',
} satisfies Metadata

const Page: FunctionComponent = () => (
  <Content>
    <SourceList />
  </Content>
)

export default Page
