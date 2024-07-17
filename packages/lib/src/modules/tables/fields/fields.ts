import { TFBase, TFExtended } from './shared';

export class TFString extends TFExtended<{
  maxLength?: number;
  minLength?: number;
  pattern?: RegExp;
}> {
  public type = 'string' as const;

  constructor(
    public values: {
      maxLength?: number;
      minLength?: number;
      pattern?: RegExp;
    } = {},
  ) {
    super('string' satisfies TFString['type'], values);
  }

  maxLength(max: number) {
    this.values.maxLength = max;
    return this;
  }

  minLength(min: number) {
    this.values.minLength = min;
    return this;
  }

  pattern(pattern: RegExp) {
    this.values.pattern = pattern;
    return this;
  }
}

export class TFNumber extends TFExtended<{
  max?: number;
  min?: number;
}> {
  public type = 'number' as const;

  constructor(
    public values: {
      max?: number;
      min?: number;
    } = {},
  ) {
    super('number' satisfies TFNumber['type'], values);
  }

  max(max: number) {
    this.values.max = max;
    return this;
  }

  min(min: number) {
    this.values.min = min;
    return this;
  }
}

export class TFBoolean extends TFBase {
  public type = 'boolean' as const;

  constructor() {
    super('boolean' satisfies TFBoolean['type']);
  }
}

export class TFDate extends TFExtended<Record<string, never>> {
  public type = 'date' as const;

  constructor() {
    super('date' satisfies TFDate['type'], {});
  }
}

export class TFEmail extends TFExtended<{
  except: string[];
  only: string[];
}> {
  public type = 'email' as const;

  constructor(
    public values: {
      except: string[];
      only: string[];
    } = {
      except: [],
      only: [],
    },
  ) {
    super('email' satisfies TFEmail['type'], values);
  }

  except(emails: string[]) {
    this.values.except = emails;
    return this;
  }

  only(emails: string[]) {
    this.values.only = emails;
    return this;
  }
}

export class TFUrl extends TFExtended<{
  except: string[];
  only: string[];
}> {
  public type = 'url' as const;

  constructor(
    public values: {
      except: string[];
      only: string[];
    } = {
      except: [],
      only: [],
    },
  ) {
    super('url' satisfies TFUrl['type'], values);
  }

  except(urls: string[]) {
    this.values.except = urls;
    return this;
  }

  only(urls: string[]) {
    this.values.only = urls;
    return this;
  }
}

export class TFSelect extends TFExtended<{
  options: string[];
  maxSelected?: number;
  minSelected?: number;
}> {
  public type = 'select' as const;

  constructor(
    public values: {
      options: string[];
      maxSelected?: number;
      minSelected?: number;
    },
  ) {
    super('select' satisfies TFSelect['type'], values);
  }

  maxSelected(max: number) {
    this.values.maxSelected = max;
    return this;
  }

  minSelected(min: number) {
    this.values.minSelected = min;
    return this;
  }
}

interface TFRelationValues extends Record<string, unknown> {
  table: string;
  cascadeDelete: boolean;
}

export class TFRelationSingle extends TFExtended<TFRelationValues> {
  public type = 'relationSingle' as const;

  constructor(public values: TFRelationValues) {
    super('relationSingle' satisfies TFRelationSingle['type'], values);
  }

  cascadeDelete() {
    this.values.cascadeDelete = true;
    return this;
  }
}

export class TFRelationMany extends TFExtended<
  TFRelationValues & {
    minSelected?: number;
    maxSelected?: number;
  }
> {
  public type = 'relationMany' as const;

  constructor(
    public values: TFRelationValues & {
      minSelected?: number;
      maxSelected?: number;
    },
  ) {
    super('relationMany' satisfies TFRelationMany['type'], values);
  }

  cascadeDelete() {
    this.values.cascadeDelete = true;
    return this;
  }

  minSelected(min: number) {
    this.values.minSelected = min;
    return this;
  }

  maxSelected(max: number) {
    this.values.maxSelected = max;
    return this;
  }
}
