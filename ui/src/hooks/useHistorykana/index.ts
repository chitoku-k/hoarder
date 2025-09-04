import historykana from 'historykana'

const options = {
  kanaRegexp: /^[\u0020\u3000ぁ-ゔー]*[nｎ]?$/,
}

function preprocess(s: string): string {
  return s.replace(/う゛/, 'ゔ')
}

function postprocess(s: string): string {
  return s.replace(/[nｎ]$/, 'ん')
}

function extract(history: string[]): string {
  return postprocess(historykana(history.map(preprocess), options))
}

export function useHistorykana(): (history: string[]) => string {
  return extract
}
