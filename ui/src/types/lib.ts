declare global {
  interface ArrayConstructor {
    fromAsync<T>(asyncIterable: AsyncIterable<T>): Promise<Array<T>>
    fromAsync<T>(iterable: Iterable<T> | ArrayLike<T>): Promise<Array<Awaited<T>>>
    fromAsync<T, U>(asyncIterable: AsyncIterable<T>, mapFn: (v: T, k: number) => U, thisArg?: any): Promise<Array<Awaited<U>>>
    fromAsync<T, U>(iterable: Iterable<T> | ArrayLike<T>, mapFn: (v: Awaited<T>, k: number) => U, thisArg?: any): Promise<Array<Awaited<U>>>
  }
}

export {}
