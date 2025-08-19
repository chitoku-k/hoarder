import { registerApolloClient } from '@apollo/client-integration-nextjs'

import { makeClient } from '@/graphql'

export const { query } = registerApolloClient(makeClient)
