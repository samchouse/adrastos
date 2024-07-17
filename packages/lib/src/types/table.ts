import type { AndNullable } from '../types';

export type Field = AndNullable<
  | StringField
  | NumberField
  | BooleanField
  | DateField
  | EmailField
  | UrlField
  | SelectField
  | RelationSingleField
  | RelationManyField
>;

interface BaseField {
  name: string;
}

interface ExtendedField extends BaseField {
  isUnique: boolean;
  isRequired: boolean;
}

export interface StringField extends ExtendedField {
  type: 'string';
  pattern?: string;
  minLength?: number;
  maxLength?: number;
}

export interface NumberField extends ExtendedField {
  min?: number;
  max?: number;
  type: 'number';
}

export interface BooleanField extends BaseField {
  type: 'boolean';
}

export interface DateField extends ExtendedField {
  type: 'date';
}

export interface EmailField extends ExtendedField {
  type: 'email';
  only: string[];
  except: string[];
}

export interface UrlField extends ExtendedField {
  type: 'url';
  only: string[];
  except: string[];
}

export interface SelectField extends ExtendedField {
  type: 'select';
  options: string[];
  minSelected?: number;
  maxSelected?: number;
}

interface RelationFieldBase extends ExtendedField {
  table: string;
  type: 'relation';
  cascadeDelete: boolean;
}

export interface RelationSingleField extends RelationFieldBase {
  target: 'single';
}

export interface RelationManyField extends RelationFieldBase {
  target: 'many';
  minSelected?: number;
  maxSelected?: number;
}

export type FieldCrud = FieldUpdate | FieldDelete;

export interface FieldUpdate {
  name: string;
  field: Field;
  action: 'create' | 'update';
}

export interface FieldDelete {
  name: string;
  action: 'delete';
}
