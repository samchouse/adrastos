export abstract class TFBase {
  constructor(
    public type:
      | 'string'
      | 'number'
      | 'boolean'
      | 'date'
      | 'email'
      | 'url'
      | 'select'
      | 'relationSingle'
      | 'relationMany',
  ) {}
}

export class TFExtended<T extends Record<string, unknown>> extends TFBase {
  constructor(
    public type: TFBase['type'],
    public values: T,
    public modifiers: readonly ('optional' | 'unique')[] = [],
  ) {
    super(type);
  }

  optional() {
    return new TFOptional<this>(this.type, this.values, this.modifiers);
  }

  unique() {
    return new TFUnique<this>(this.type, this.values, this.modifiers);
  }
}

export class TFOptional<
  U extends TFExtended<Record<string, unknown>>,
> extends TFExtended<U['values']> {
  private _name = 'optional';

  constructor(
    public type: U['type'],
    values: U['values'],
    modifiers: U['modifiers'],
  ) {
    super(type, values, [...modifiers, 'optional']);
  }
}

export class TFUnique<
  U extends TFExtended<Record<string, unknown>>,
> extends TFExtended<U['values']> {
  private _name = 'unique';

  constructor(
    public type: U['type'],
    values: U['values'],
    modifiers: U['modifiers'],
  ) {
    super(type, values, [...modifiers, 'unique']);
  }
}
