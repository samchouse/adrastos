import { TBase, TExtended } from './shared';

export class TString extends TExtended {
  private _name = 'string';

  constructor(
    public values: {
      maxLength?: number;
      minLength?: number;
      pattern?: RegExp;
    } = {},
  ) {
    super('string', [], values);
  }

  maxLength(max: number): TString {
    this.values.maxLength = max;
    return this;
  }

  minLength(min: number): TString {
    this.values.minLength = min;
    return this;
  }

  pattern(pattern: RegExp): TString {
    this.values.pattern = pattern;
    return this;
  }
}

export class TNumber extends TExtended {
  private _name = 'string';

  constructor(public values: { max?: number; min?: number } = {}) {
    super('string', [], values);
  }

  max(max: number): TNumber {
    this.values.max = max;
    return this;
  }

  min(min: number): TNumber {
    this.values.min = min;
    return this;
  }
}

export class TBoolean extends TBase {
  private _name = 'boolean';

  constructor() {
    super('boolean', [], {});
  }
}

export class TDate extends TExtended {
  private _name = 'date';

  constructor() {
    super('date', [], {});
  }
}

export class TEmail extends TExtended {
  private _name = 'email';

  constructor(
    public values: { except: string[]; only: string[] } = {
      except: [],
      only: [],
    },
  ) {
    super('email', [], values);
  }

  except(emails: string[]): TEmail {
    this.values.except = emails;
    return this;
  }

  only(emails: string[]): TEmail {
    this.values.only = emails;
    return this;
  }
}

export class TUrl extends TExtended {
  private _name = 'url';

  constructor(
    public values: { except: string[]; only: string[] } = {
      except: [],
      only: [],
    },
  ) {
    super('url', [], values);
  }

  except(urls: string[]): TUrl {
    this.values.except = urls;
    return this;
  }

  only(urls: string[]): TUrl {
    this.values.only = urls;
    return this;
  }
}

export class TSelect extends TExtended {
  private _name = 'select';

  constructor(
    public values: {
      options: string[];
      maxSelected?: number;
      minSelected?: number;
    },
  ) {
    super('select', [], values);
  }

  maxSelected(max: number): TSelect {
    this.values.maxSelected = max;
    return this;
  }

  minSelected(min: number): TSelect {
    this.values.minSelected = min;
    return this;
  }
}

interface TRelationValues extends Record<string, unknown> {
  table: string;
  cascadeDelete?: boolean;
}

export class TRelationSingle extends TExtended {
  private _name = 'relationSingle';

  constructor(public values: TRelationValues) {
    super('relation', ['single'], values);
  }
}

export class TRelationMany extends TExtended {
  private _name = 'relationMany';

  constructor(
    public values: TRelationValues & {
      minSelected?: number;
      maxSelected?: number;
    },
  ) {
    super('relation', ['multi'], values);
  }

  minSelected(min: number): TRelationMany {
    this.values.minSelected = min;
    return this;
  }

  maxSelected(max: number): TRelationMany {
    this.values.maxSelected = max;
    return this;
  }
}
