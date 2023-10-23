import { z } from 'zod';

type ZExtend<T> = T extends TOptional<infer U>
  ? z.ZodOptional<ZExtend<U>>
  : T extends TUnique<infer U>
  ? ZExtend<U>
  : T extends TString
  ? z.ZodString
  : never;

class Table<T extends Record<string, TBase>> {
  constructor(private shape: T) {}

  schema(): z.ZodObject<{
    [Key in keyof T]: ZExtend<T[Key]>;
  }> {
    return z.object(
      Object.keys(this.shape).reduce(
        (acc, key) => {
          const setModifiers = (type: z.ZodTypeAny) => {
            if (this.shape[key].modifiers.includes('optional'))
              type = type.optional();

            return type;
          };

          switch (this.shape[key].type) {
            case 'string':
              acc[key] = z.string();
              break;
          }

          acc[key] = setModifiers(acc[key]);
          return acc;
        },
        {} as Record<string, z.ZodTypeAny>,
      ),
    ) as any;
  }
}

class TBuilder {
  string(): TString {
    return new TString();
  }
}

abstract class TBase {
  constructor(
    public type: 'string' | 'number' | 'boolean',
    public modifiers: string[],
    public values: Record<string, unknown>,
  ) {}

  optional(): TOptional<this> {
    return new TOptional(this.type, this.modifiers, this.values);
  }

  unique(): TUnique<this> {
    return new TUnique(this.type, this.modifiers, this.values);
  }
}

class TOptional<T extends TBase> extends TBase {
  private _name = 'optional';

  constructor(type: T['type'], modifiers: T['modifiers'], values: T['values']) {
    super(type, [...modifiers, 'optional'], values);
  }
}

class TUnique<T extends TBase> extends TBase {
  private _name = 'unique';

  constructor(type: T['type'], modifiers: T['modifiers'], values: T['values']) {
    super(type, [...modifiers, 'unique'], values);
  }
}

class TString extends TBase {
  private _name = 'string';

  constructor(public values: { maxLength?: number; minLength?: number } = {}) {
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
}

const table = <T extends Record<string, TBase>>(
  name: string,
  shape: (builder: TBuilder) => T,
): Table<T> => new Table(shape(new TBuilder()));

const tbl = table('recipes', (b) => ({
  name: b.string().maxLength(20).minLength(20).unique().optional(),
}));

console.log(tbl.schema().parse({}));
