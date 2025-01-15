import fs from 'node:fs/promises'
import { parseArgs } from 'node:util'

const { values } = parseArgs({
  options: {
    version: {
      type: 'boolean',
    },
  },
})
if (values.version) {
  const { default: app } = await import('./package.json', {
    with: {
      type: 'json',
    },
  })
  process.stdout.write(`${app.name} ${app.version}\n`)
  process.exit()
}

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
]

for (const filename of filenames) {
  const content = await fs.readFile(filename, { encoding: 'utf-8' })
  await fs.writeFile(
    filename,
    envs.reduce((s, { substitute }) => substitute(s), content),
  )
}

await import('./server.js')
