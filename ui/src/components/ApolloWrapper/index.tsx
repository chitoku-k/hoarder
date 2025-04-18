'use client'

import type { FunctionComponent, ReactNode } from 'react'
import { ApolloNextAppProvider } from '@apollo/experimental-nextjs-app-support'

import { makeClient } from '@/graphql'

const ApolloWrapper: FunctionComponent<ApolloWrapperProps> = ({
  children,
}) => (
  <ApolloNextAppProvider makeClient={makeClient}>
    {children}
  </ApolloNextAppProvider>
)

export interface ApolloWrapperProps {
  children: ReactNode
}

export default ApolloWrapper
