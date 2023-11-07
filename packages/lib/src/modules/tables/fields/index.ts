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
import { TBase, TOptional, TUnique } from './shared';

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

export class Table<T extends Record<string, TBase>> {
  constructor(private shape: T) {}

  schema(): z.ZodObject<{
    [Key in keyof T]: ZExtend<T[Key]>;
  }> {
    return z.object(
      Object.keys(this.shape).reduce(
        (acc, key) => {
          const setModifiers = (type: z.ZodTypeAny) => {
            if (this.shape[key].modifiers.includes('optional'))
              type = type.optional();

            return type;
          };

          switch (this.shape[key].type) {
            case 'string':
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
              acc[key] = z.array(z.string());
              break;
            case 'relation':
              acc[key] = z.string();
              break;
          }

          acc[key] = setModifiers(acc[key]);
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

const table = <T extends Record<string, TBase>>(
  name: string,
  shape: (builder: TBuilder) => T,
): Table<T> => new Table(shape(new TBuilder()));

const tbl = table('recipes', (b) => ({
  name: b.string().maxLength(20).minLength(20).unique().optional(),
  ingredients: b.number().max(10).min(1).optional(),
  sdf: b.boolean(),
  asdsdfs: b.relation({ table: 'sdf', target: 'many' }).maxSelected(10),
}));

console.log(tbl.schema().parse({}));
