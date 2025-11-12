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

await import('./server.js')
