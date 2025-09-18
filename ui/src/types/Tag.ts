export interface Tag {
  readonly id: string
  readonly name: string
  readonly kana: string
  readonly aliases: readonly string[]
  readonly parent?: Tag | null
  readonly children?: readonly Tag[]
}
