import { z } from 'zod';

import type { Field } from '../../../types';
import {
  TFBoolean,
  TFDate,
  TFEmail,
  TFNumber,
  TFRelationMany,
  TFRelationSingle,
  TFSelect,
  TFString,
  TFUrl,
} from './fields';
import type { TFOptional, TFUnique } from './shared';

export interface BaseData {
  id: string;
  createdAt: Date;
  updatedAt: Date;
}

type Data = Record<
  string,
  string | number | boolean | string[] | Date | undefined
>;
export type Row<T extends Data = Data> = T & {
  id: string;
  createdAt: Date;
  updatedAt: Date;
};

type ZExtend<T> = T extends TFOptional<infer U>
  ? z.ZodOptional<ZExtend<U>>
  : T extends TFUnique<infer U>
    ? ZExtend<U>
    : T extends TFString
      ? z.ZodString
      : T extends TFNumber
        ? z.ZodNumber
        : T extends TFBoolean
          ? z.ZodBoolean
          : T extends TFDate
            ? z.ZodDate
            : T extends TFEmail
              ? z.ZodString
              : T extends TFUrl
                ? z.ZodString
                : T extends TFSelect
                  ? z.ZodArray<z.ZodString>
                  : T extends TFRelationSingle
                    ? z.ZodString
                    : T extends TFRelationMany
                      ? z.ZodArray<z.ZodString>
                      : never;

export type TField =
  | TFString
  | TFNumber
  | TFBoolean
  | TFDate
  | TFEmail
  | TFUrl
  | TFSelect
  | TFRelationSingle
  | TFRelationMany;

type TFOptionalWrapper<
  T extends
    | Exclude<TField, TFBoolean>
    | TFUniqueWrapper<Exclude<TField, TFBoolean>>,
> = T extends unknown ? TFOptional<T> : never;

type TFUniqueWrapper<
  T extends
    | Exclude<TField, TFBoolean>
    | TFOptionalWrapper<Exclude<TField, TFBoolean>>,
> = T extends unknown ? TFUnique<T> : never;

export type TFWithModifiers =
  | TField
  | TFOptionalWrapper<
      Exclude<TField, TFBoolean> | TFUniqueWrapper<Exclude<TField, TFBoolean>>
    >
  | TFUniqueWrapper<
      Exclude<TField, TFBoolean> | TFOptionalWrapper<Exclude<TField, TFBoolean>>
    >;

export type TInfer<T> = T extends Table
  ? Row<z.infer<ReturnType<T['schema']>>>
  : never;

export class Table<
  T extends string = string,
  U extends Record<string, TFWithModifiers> = Record<string, TFWithModifiers>,
> {
  constructor(
    public name: T,
    private shape: U,
  ) {}

  requestBody() {
    return Object.keys(this.shape).reduce(
      (acc, key) => {
        const field = this.shape[key];
        switch (field.type) {
          case 'string':
            acc.fields.push({
              name: key,
              type: 'string',
              minLength: field.values.minLength,
              maxLength: field.values.maxLength,
              pattern: field.values.pattern?.toString(),
              isUnique: field.modifiers.includes('unique'),
              isRequired: !field.modifiers.includes('optional'),
            });
            break;
          case 'number':
            acc.fields.push({
              name: key,
              type: 'number',
              min: field.values.min,
              max: field.values.max,
              isUnique: field.modifiers.includes('unique'),
              isRequired: !field.modifiers.includes('optional'),
            });
            break;
          case 'boolean':
            acc.fields.push({
              name: key,
              type: 'boolean',
            });
            break;
          case 'date':
            acc.fields.push({
              name: key,
              type: 'date',
              isUnique: field.modifiers.includes('unique'),
              isRequired: !field.modifiers.includes('optional'),
            });
            break;
          case 'email':
            acc.fields.push({
              name: key,
              type: 'email',
              only: field.values.only,
              except: field.values.except,
              isUnique: field.modifiers.includes('unique'),
              isRequired: !field.modifiers.includes('optional'),
            });
            break;
          case 'url':
            acc.fields.push({
              name: key,
              type: 'url',
              only: field.values.only,
              except: field.values.except,
              isUnique: field.modifiers.includes('unique'),
              isRequired: !field.modifiers.includes('optional'),
            });
            break;
          case 'select':
            acc.fields.push({
              name: key,
              type: 'select',
              options: field.values.options,
              minSelected: field.values.minSelected,
              maxSelected: field.values.maxSelected,
              isUnique: field.modifiers.includes('unique'),
              isRequired: !field.modifiers.includes('optional'),
            });
            break;
          case 'relationSingle':
            acc.fields.push({
              name: key,
              type: 'relation',
              target: 'single',
              table: field.values.table,
              cascadeDelete: field.values.cascadeDelete,
              isUnique: field.modifiers.includes('unique'),
              isRequired: !field.modifiers.includes('optional'),
            });
            break;
          case 'relationMany':
            acc.fields.push({
              name: key,
              target: 'many',
              type: 'relation',
              table: field.values.table,
              maxSelected: field.values.maxSelected,
              minSelected: field.values.minSelected,
              cascadeDelete: field.values.cascadeDelete,
              isUnique: field.modifiers.includes('unique'),
              isRequired: !field.modifiers.includes('optional'),
            });
            break;
        }

        return acc;
      },
      {
        name: this.name,
        fields: [] as Field[],
        permissions: {
          view: null,
          create: null,
          update: null,
          delete: null,
        } as {
          view: string | null;
          create: string | null;
          update: string | null;
          delete: string | null;
        },
      },
    );
  }

  schema(): z.ZodObject<{
    [Key in keyof U]: ZExtend<U[Key]>;
  }> {
    return z.object(
      Object.keys(this.shape).reduce<Record<string, z.ZodTypeAny>>(
        (acc, key) => {
          const field = this.shape[key];
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

          if (field.type !== 'boolean' && field.modifiers.includes('optional'))
            acc[key] = acc[key].optional();

          return acc;
        },
        {},
      ),
    ) as z.ZodObject<{
      [Key in keyof U]: ZExtend<U[Key]>;
    }>;
  }
}

class TBuilder {
  string() {
    return new TFString();
  }

  number() {
    return new TFNumber();
  }

  boolean() {
    return new TFBoolean();
  }

  date() {
    return new TFDate();
  }

  email() {
    return new TFEmail();
  }

  url() {
    return new TFUrl();
  }

  select({ options }: { options: string[] }) {
    return new TFSelect({ options });
  }

  relation<T extends { target: 'single' | 'many' }>({
    table,
    target,
  }: {
    table: string;
  } & T) {
    return (
      target === 'single'
        ? new TFRelationSingle({
            table,
            cascadeDelete: false,
          })
        : new TFRelationMany({
            table,
            cascadeDelete: false,
          })
    ) as T['target'] extends 'single' ? TFRelationSingle : TFRelationMany;
  }
}

export function table<
  T extends string,
  U extends Record<string, TFWithModifiers>,
>(name: T, shape: (builder: TBuilder) => U): Table<T, U> {
  return new Table(name, shape(new TBuilder()));
}
