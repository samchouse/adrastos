export type Field =
  | StringField
  | NumberField
  | BooleanField
  | DateField
  | EmailField
  | UrlField
  | SelectField
  | RelationField;

interface BaseField {
  name: string;
}

interface ExtendedField extends BaseField {
  isRequired: boolean;
  isUnique: boolean;
}

export interface StringField extends ExtendedField {
  type: 'string';
  minLength?: number;
  maxLength?: number;
  pattern?: string;
}

export interface NumberField extends ExtendedField {
  type: 'number';
  min?: number;
  max?: number;
}

export interface BooleanField extends BaseField {
  type: 'boolean';
}

export interface DateField extends ExtendedField {
  type: 'date';
}

export interface EmailField extends ExtendedField {
  type: 'email';
  except: string[];
  only: string[];
}

export interface UrlField extends ExtendedField {
  type: 'url';
  except: string[];
  only: string[];
}

export interface SelectField extends ExtendedField {
  type: 'select';
  options: string[];
  minSelected?: number;
  maxSelected?: number;
}

export interface RelationField extends ExtendedField {
  type: 'relation';
  table: string;
  target: 'single' | 'many';
  minSelected?: number;
  maxSelected?: number;
  cascadeDelete: boolean;
}

export interface UpdateField<T extends BaseField> {
  name: string;
  action: 'create' | 'update' | 'delete';
  field: T;
}
