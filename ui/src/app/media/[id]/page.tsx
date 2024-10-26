import type { FunctionComponent } from 'react'
import type { Metadata } from 'next'

import Content from '@/components/Content'
import MediumItem from '@/components/MediumItem'

export const metadata: Metadata = {
  title: 'メディア',
}

const Page: FunctionComponent<PageProps> = async ({
  params,
}) => {
  const { id } = await params
  return (
    <Content size={12}>
      <MediumItem id={id} />
    </Content>
  )
}

export interface Params {
  id: string
}

export interface PageProps {
  params: Promise<Params>
}

export default Page
