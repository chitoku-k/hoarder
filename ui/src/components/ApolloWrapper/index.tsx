'use client'

import type { FunctionComponent, ReactNode } from 'react'
import { ApolloNextAppProvider } from '@apollo/client-integration-nextjs'

import { makeClient } from '@/graphql'

const ApolloWrapper: FunctionComponent<ApolloWrapperProps> = ({
  children,
}) => (
  <ApolloNextAppProvider makeClient={makeClient}>
    {children}
  </ApolloNextAppProvider>
)

export interface ApolloWrapperProps {
  readonly children: ReactNode
}

export default ApolloWrapper
