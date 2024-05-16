import type { FunctionComponent } from 'react'

import Content from '@/components/Content'
import MediumList from '@/components/MediumList'

const Page: FunctionComponent<PageProps> = () => (
  <Content>
    <MediumList number={48} />
  </Content>
)

export interface PageProps {
  searchParams: {}
}

export default Page
