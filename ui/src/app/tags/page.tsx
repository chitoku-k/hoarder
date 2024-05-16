import type { FunctionComponent } from 'react'
import type { Metadata } from 'next'

import Content from '@/components/Content'
import TagList from '@/components/TagList'

export const metadata: Metadata = {
  title: 'タグ',
}

const Page: FunctionComponent = ({
}) => (
  <Content>
    <TagList />
  </Content>
)

export default Page
