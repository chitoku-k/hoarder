import type { CodegenConfig } from '@graphql-codegen/cli'

const config: CodegenConfig = {
  schema: '../schema/hoarder.gql',
  config: {
    scalars: {
      DateTime: 'string',
      JSON: 'unknown',
      Upload: 'File',
      UUID: 'string',
    },
    skipTypename: true,
    strictScalars: true,
  },
  documents: [
    'src/hooks/**/*.gql',
  ],
  ignoreNoDocuments: true,
  generates: {
    'src/hooks/': {
      preset: 'near-operation-file',
      presetConfig: {
        baseTypesPath: 'types.generated.ts',
      },
      plugins: [
        'typescript-operations',
        'typed-document-node',
      ],
    },
    'src/hooks/types.generated.ts': {
      plugins: [
        'typescript',
      ],
      config: {
        enumsAsConst: true,
      },
    },
  },
}

export default config
