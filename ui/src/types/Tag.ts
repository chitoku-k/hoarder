export interface Tag {
  id: string
  name: string
  kana: string
  aliases: string[]
  parent?: Tag | null
  children?: Tag[]
}
