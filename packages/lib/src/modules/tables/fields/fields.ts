import { TBase, TExtended } from './shared';

export class TString extends TExtended {
  public static type = 'string' as const;

  constructor(
    public values: {
      maxLength?: number;
      minLength?: number;
      pattern?: RegExp;
    } = {},
  ) {
    super(TString.type, [], values);
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

export class TNumber extends TExtended {
  public static type = 'number' as const;

  constructor(public values: { max?: number; min?: number } = {}) {
    super(TNumber.type, [], values);
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

export class TBoolean extends TBase {
  public static type = 'boolean' as const;
  public static modifiers = [] as const;

  constructor() {
    super(TBoolean.type, [], {});
  }
}

export class TDate extends TExtended {
  public static type = 'date' as const;

  constructor() {
    super(TDate.type, [], {});
  }
}

export class TEmail extends TExtended {
  public static type = 'email' as const;

  constructor(
    public values: { except: string[]; only: string[] } = {
      except: [],
      only: [],
    },
  ) {
    super(TEmail.type, [], values);
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

export class TUrl extends TExtended {
  public static type = 'url' as const;

  constructor(
    public values: { except: string[]; only: string[] } = {
      except: [],
      only: [],
    },
  ) {
    super(TUrl.type, [], values);
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

export class TSelect extends TExtended {
  public static type = 'select' as const;

  constructor(
    public values: {
      options: string[];
      maxSelected?: number;
      minSelected?: number;
    },
  ) {
    super(TSelect.type, [], values);
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

interface TRelationValues extends Record<string, unknown> {
  table: string;
  cascadeDelete?: boolean;
}

export class TRelationSingle extends TExtended {
  public static type = 'relationSingle' as const;

  constructor(public values: TRelationValues) {
    super(TRelationSingle.type, [], values);
  }
}

export class TRelationMany extends TExtended {
  public static type = 'relationMany' as const;

  constructor(
    public values: TRelationValues & {
      minSelected?: number;
      maxSelected?: number;
    },
  ) {
    super(TRelationMany.type, [], values);
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
