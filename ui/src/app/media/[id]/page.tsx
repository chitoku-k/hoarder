import type { FunctionComponent } from 'react'
import type { Metadata } from 'next'

import Content from '@/components/Content'
import MediumItem from '@/components/MediumItem'

export const metadata: Metadata = {
  title: 'メディア',
}

const Page: FunctionComponent<PageProps> = ({
  params: {
    id,
  },
}) => (
  <Content xs={12}>
    <MediumItem id={id} />
  </Content>
)

export interface PageProps {
  params: {
    id: string
  }
}

export default Page
