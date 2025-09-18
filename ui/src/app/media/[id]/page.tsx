import type { FunctionComponent } from 'react'
import type { Metadata } from 'next'

import Content from '@/components/Content'
import MediumItem from '@/components/MediumItem'

export const metadata = {
  title: 'メディア',
} satisfies Metadata

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
  readonly id: string
}

export interface PageProps {
  readonly params: Promise<Params>
}

export default Page
