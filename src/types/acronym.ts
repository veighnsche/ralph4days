const ACRONYM_PATTERN = /^[A-Z0-9]{4}$/

declare const acronymBrand: unique symbol

export type Acronym = string & { readonly [acronymBrand]: 'Acronym' }

type StringToTuple<S extends string, T extends unknown[] = []> = string extends S
  ? T
  : S extends `${infer _}${infer Rest}`
    ? StringToTuple<Rest, [...T, unknown]>
    : T

type StringLength<S extends string> = StringToTuple<S>['length']

type FourChars<S extends string> = StringLength<S> extends 4 ? S : never
type UppercaseLiteral<S extends string> = Uppercase<S> extends S ? S : never

export function isAcronym(value: string): value is Acronym {
  return ACRONYM_PATTERN.test(value)
}

export function toAcronym(value: string): Acronym {
  if (!isAcronym(value)) {
    throw new Error(`Invalid acronym "${value}". Expected exactly 4 uppercase alphanumeric characters.`)
  }
  return value as Acronym
}

export function acronym<const S extends string>(value: FourChars<UppercaseLiteral<S>>): Acronym {
  return toAcronym(value)
}
