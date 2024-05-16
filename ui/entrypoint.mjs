import fs from 'node:fs/promises'

const filenames = [
  '.next/required-server-files.json',
  '.next/routes-manifest.json',
  'server.js',
]

const envs = [
  {
    substitute(s) {
      if (!process.env.API_URL) {
        throw new Error('API_URL must be set')
      }
      return s.replace(
        /https:\/\/__HOARDER_API_URL__/giu,
        process.env.API_URL,
      )
    },
  },
  {
    substitute(s) {
      if (!process.env.PUBLIC_URL) {
        throw new Error('PUBLIC_URL must be set')
      }
      return s.replace(
        /__HOARDER_PUBLIC_URL__/giu,
        process.env.PUBLIC_URL.replace(/^https?:\/\//, ''),
      )
    },
  },
]

for (const filename of filenames) {
  const content = await fs.readFile(filename, { encoding: 'utf-8' })
  await fs.writeFile(
    filename,
    envs.reduce((s, { substitute }) => substitute(s), content),
  )
}

await import('./server.js')
