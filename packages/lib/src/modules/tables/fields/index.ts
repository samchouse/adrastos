import { z } from 'zod';

import {
  TBoolean,
  TDate,
  TEmail,
  TNumber,
  TRelationMany,
  TRelationSingle,
  TSelect,
  TString,
  TUrl,
} from './fields';
import { TOptional, TUnique } from './shared';

export * from './fields';
export * from './shared';

export type TInfer<T> = T extends Table<infer _>
  ? z.infer<ReturnType<T['schema']>>
  : never;

type RelationBuilderReturn<T extends { target: 'single' | 'many' }> =
  T['target'] extends 'single' ? TRelationSingle : TRelationMany;

type ZExtend<T> = T extends TOptional<infer U>
  ? z.ZodOptional<ZExtend<U>>
  : T extends TUnique<infer U>
  ? ZExtend<U>
  : T extends TString
  ? z.ZodString
  : T extends TNumber
  ? z.ZodNumber
  : T extends TBoolean
  ? z.ZodBoolean
  : T extends TDate
  ? z.ZodDate
  : T extends TEmail
  ? z.ZodString
  : T extends TUrl
  ? z.ZodString
  : T extends TSelect
  ? z.ZodArray<z.ZodString, 'many'>
  : T extends TRelationSingle
  ? z.ZodString
  : T extends TRelationMany
  ? z.ZodArray<z.ZodString, 'many'>
  : never;

type TT =
  | TString
  | TNumber
  | TBoolean
  | TDate
  | TEmail
  | TUrl
  | TSelect
  | TRelationSingle
  | TRelationMany;

export type TTE =
  | TT
  | TOptional<Exclude<TT, TBoolean>>
  | TUnique<Exclude<TT, TBoolean>>;

export class Table<T extends Record<string, TTE>> {
  constructor(private shape: T) {}

  schema(): z.ZodObject<{
    [Key in keyof T]: ZExtend<T[Key]>;
  }> {
    return z.object(
      Object.keys(this.shape).reduce(
        (acc, key) => {
          const field = this.shape[key];
          if (field.type === 'email') field.values;
          switch (field.type) {
            case 'string':
            case 'relationSingle':
              acc[key] = z.string();
              break;
            case 'number':
              acc[key] = z.number();
              break;
            case 'boolean':
              acc[key] = z.boolean();
              break;
            case 'date':
              acc[key] = z.date();
              break;
            case 'email':
              acc[key] = z.string().email();
              break;
            case 'url':
              acc[key] = z.string().url();
              break;
            case 'select':
            case 'relationMany':
              acc[key] = z.array(z.string());
              break;
          }

          if (field.modifiers.includes('optional'))
            acc[key] = acc[key].optional();

          return acc;
        },
        {} as Record<string, z.ZodTypeAny>,
      ),
    ) as z.ZodObject<{
      [Key in keyof T]: ZExtend<T[Key]>;
    }>;
  }
}

export class TBuilder {
  string(): TString {
    return new TString();
  }
  number(): TNumber {
    return new TNumber();
  }
  boolean(): TBoolean {
    return new TBoolean();
  }
  date(): TDate {
    return new TDate();
  }
  email(): TEmail {
    return new TEmail();
  }
  url(): TUrl {
    return new TUrl();
  }
  select({ options }: { options: string[] }): TSelect {
    return new TSelect({ options });
  }
  relation<T extends { target: 'single' | 'many' }>({
    table,
    target,
  }: {
    table: string;
  } & T): RelationBuilderReturn<T> {
    return (
      target === 'single'
        ? new TRelationSingle({ table })
        : new TRelationMany({ table })
    ) as RelationBuilderReturn<T>;
  }
}

export const table = <T extends Record<string, TTE>>(
  name: string,
  shape: (builder: TBuilder) => T,
): Table<T> => new Table(shape(new TBuilder()));
