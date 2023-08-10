interface Field {
  name: string;
}

export interface StringField extends Field {
  minLength?: number;
  maxLength?: number;
  pattern?: string;
  isRequired?: boolean;
  isUnique?: boolean;
}

export interface StringField extends Field {
  minLength?: number;
  maxLength?: number;
  pattern?: string;
  isRequired?: boolean;
  isUnique?: boolean;
}

export interface NumberField extends Field {
  min?: number;
  max?: number;
  isRequired: boolean;
  isUnique: boolean;
}

export interface BooleanField extends Field {}

export interface DateField extends Field {
  isRequired: boolean;
  isUnique: boolean;
}

export interface EmailField extends Field {
  except: string[];
  only: string[];
  isRequired: boolean;
  isUnique: boolean;
}

export interface UrlField extends Field {
  except: string[];
  only: string[];
  isRequired: boolean;
  isUnique: boolean;
}

export interface SelectField extends Field {
  options: string[];
  minSelected?: number;
  maxSelected?: number;
  isRequired: boolean;
  isUnique: boolean;
}

export interface RelationField extends Field {
  table: string;
  type: 'single' | 'many';
  minSelected?: number;
  maxSelected?: number;
  cascadeDelete: boolean;
  isRequired: boolean;
  isUnique: boolean;
}

export interface UpdateField<T extends Field> {
  name: string;
  action: 'create' | 'update' | 'delete';
  field: T;
}
