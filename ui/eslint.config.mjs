import { defineConfig } from 'eslint/config'
import eslint from '@eslint/js'
import nextVitals from 'eslint-config-next/core-web-vitals'
import nextTs from 'eslint-config-next/typescript'
import stylistic from '@stylistic/eslint-plugin'
import tseslint from 'typescript-eslint'

const configs = defineConfig(
  eslint.configs.recommended,
  stylistic.configs.recommended,
  ...nextVitals,
  ...nextTs,
  {
    ignores: [
      '.storybook/**',
      '**/*.generated.*',
    ],
  },
  {
    rules: {
      'jsx-a11y/alt-text': 'off',
      '@stylistic/array-bracket-spacing': [ 'error', 'always' ],
      '@stylistic/arrow-parens': [ 'error', 'as-needed' ],
      '@stylistic/brace-style': [ 'error', '1tbs' ],
      '@stylistic/jsx-one-expression-per-line': 'off',
      '@stylistic/jsx-wrap-multilines': [
        'error',
        {
          prop: 'ignore',
        },
      ],
      '@stylistic/multiline-ternary': 'off',
    },
  },
  {
    files: [
      '**/*.{ts,tsx}',
    ],
    ...tseslint.configs.base,
    languageOptions: {
      ...tseslint.configs.base.languageOptions,
      parserOptions: {
        project: true,
      },
    },
    rules: {
      ...Object.assign(
        {},
        ...tseslint.configs.strictTypeChecked.map(({ rules }) => rules),
        ...tseslint.configs.stylisticTypeChecked.map(({ rules }) => rules),
      ),
      '@typescript-eslint/no-confusing-void-expression': 'off',
      '@typescript-eslint/no-misused-promises': [
        'error',
        {
          checksVoidReturn: {
            attributes: false,
          },
        },
      ],
      '@typescript-eslint/no-restricted-types': [
        'error',
        {
          types: {
            Readonly: {
              message: 'Use `readonly`, `ReadonlyMap`, or `ReadonlySet`.',
            },
          },
        },
      ],
      '@typescript-eslint/no-unused-vars': [
        'error',
        {
          ignoreRestSiblings: true,
        },
      ],
    },
  },
)

export default configs
