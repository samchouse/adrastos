import { z } from 'zod';

import { Field } from '../../../types';
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
import { TFOptional, TFUnique } from './shared';

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
  ? z.ZodArray<z.ZodString, 'many'>
  : T extends TFRelationSingle
  ? z.ZodString
  : T extends TFRelationMany
  ? z.ZodArray<z.ZodString, 'many'>
  : never;

type TField =
  | TFString
  | TFNumber
  | TFBoolean
  | TFDate
  | TFEmail
  | TFUrl
  | TFSelect
  | TFRelationSingle
  | TFRelationMany;

export type TFWithModifiers =
  | TField
  | TFOptional<Exclude<TField, TFBoolean>>
  | TFUnique<Exclude<TField, TFBoolean>>;

export type TInfer<T> = T extends Table<infer _>
  ? z.infer<ReturnType<T['schema']>>
  : never;

export class Table<T extends Record<string, TFWithModifiers>> {
  constructor(
    private name: string,
    private shape: T,
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
              minLength: field.values.minLength as number,
              maxLength: field.values.maxLength as number,
              isUnique: field.modifiers.includes('unique'),
              isRequired: !field.modifiers.includes('optional'),
            });
            break;
          case 'number':
            acc.fields.push({
              name: key,
              type: 'number',
              min: field.values.min as number,
              max: field.values.max as number,
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
              except: field.values.except as string[],
              only: field.values.only as string[],
              isUnique: field.modifiers.includes('unique'),
              isRequired: !field.modifiers.includes('optional'),
            });
            break;
          case 'url':
            acc.fields.push({
              name: key,
              type: 'url',
              except: field.values.except as string[],
              only: field.values.only as string[],
              isUnique: field.modifiers.includes('unique'),
              isRequired: !field.modifiers.includes('optional'),
            });
            break;
          case 'select':
            acc.fields.push({
              name: key,
              type: 'select',
              options: field.values.options as string[],
              minSelected: field.values.minSelected as number,
              maxSelected: field.values.maxSelected as number,
              isUnique: field.modifiers.includes('unique'),
              isRequired: !field.modifiers.includes('optional'),
            });
            break;
          case 'relationSingle':
          case 'relationMany':
            acc.fields.push({
              name: key,
              type: 'relation',
              table: field.values.table as string,
              cascadeDelete: field.values.cascadeDelete as boolean,
              target: field.type === 'relationSingle' ? 'single' : 'many',
              isRequired: !field.modifiers.includes('optional'),
              isUnique: field.modifiers.includes('unique'),
              ...(field.type === 'relationSingle' && {
                maxSelected: field.values.maxSelected as number,
                minSelected: field.values.minSelected as number,
              }),
            });
            break;
        }

        return acc;
      },
      { name: this.name, fields: [] as Field[] },
    );
  }

  schema(): z.ZodObject<{
    [Key in keyof T]: ZExtend<T[Key]>;
  }> {
    return z.object(
      Object.keys(this.shape).reduce(
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
        {} as Record<string, z.ZodTypeAny>,
      ),
    ) as z.ZodObject<{
      [Key in keyof T]: ZExtend<T[Key]>;
    }>;
  }
}

export class TBuilder {
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
        ? new TFRelationSingle({ table, cascadeDelete: false })
        : new TFRelationMany({ table, cascadeDelete: false })
    ) as T['target'] extends 'single' ? TFRelationSingle : TFRelationMany;
  }
}

export const table = <T extends Record<string, TFWithModifiers>>(
  name: string,
  shape: (builder: TBuilder) => T,
): Table<T> => new Table(name, shape(new TBuilder()));
