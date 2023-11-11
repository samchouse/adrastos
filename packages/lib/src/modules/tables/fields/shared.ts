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

export class TFExtended extends TFBase {
  constructor(
    public type: TFBase['type'],
    public values: Record<string, unknown> = {},
    public modifiers: readonly ('optional' | 'unique')[] = [],
  ) {
    super(type);
  }

  optional() {
    return new TFOptional(this.type, this.values, this.modifiers);
  }

  unique() {
    return new TFUnique(this.type, this.values, this.modifiers);
  }
}

export class TFOptional<T extends TFExtended> extends TFExtended {
  private _name = 'optional';

  constructor(type: T['type'], values: T['values'], modifiers: T['modifiers']) {
    super(type, values, [...modifiers, 'optional']);
  }
}

export class TFUnique<T extends TFExtended> extends TFExtended {
  private _name = 'unique';

  constructor(type: T['type'], values: T['values'], modifiers: T['modifiers']) {
    super(type, values, [...modifiers, 'unique']);
  }
}
