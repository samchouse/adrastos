export abstract class TBase {
  constructor(
    public type:
      | 'string'
      | 'number'
      | 'boolean'
      | 'date'
      | 'email'
      | 'url'
      | 'select'
      | 'relation',
    public modifiers: string[],
    public values: Record<string, unknown>,
  ) {}
}

export class TExtended extends TBase {
  optional(): TOptional<this> {
    return new TOptional(this.type, this.modifiers, this.values);
  }

  unique(): TUnique<this> {
    return new TUnique(this.type, this.modifiers, this.values);
  }
}

export class TOptional<T extends TExtended> extends TExtended {
  private _name = 'optional';

  constructor(type: T['type'], modifiers: T['modifiers'], values: T['values']) {
    super(type, [...modifiers, 'optional'], values);
  }
}

export class TUnique<T extends TExtended> extends TExtended {
  private _name = 'unique';

  constructor(type: T['type'], modifiers: T['modifiers'], values: T['values']) {
    super(type, [...modifiers, 'unique'], values);
  }
}
