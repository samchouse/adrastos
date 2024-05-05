import { Field, FieldUpdate } from '../../types';
import { merge } from '../../util';
import { BaseModule } from '../util';
import { Table, TFWithModifiers, TInfer } from './fields';

export { table, Table, TFWithModifiers, TInfer, TField } from './fields';

export interface CustomTable {
  id: string;
  name: string;
  fields: Field[];
  createdAt: string;
  updatedAt: string;
}

interface BaseData {
  id: string;
  createdAt: Date;
  updatedAt: Date;
}

interface UnknownData
  extends BaseData,
    Record<string, string | number | boolean | string[] | Date | undefined> {}

export class TablesModule extends BaseModule {
  public async list() {
    return this.client.json<CustomTable[]>({
      path: '/tables/list',
      method: 'GET',
      projectIdNeeded: true,
    });
  }

  public async create<T>(
    table: T extends Table<infer _, string>
      ? T
      : ReturnType<
          Table<Record<string, TFWithModifiers>, string>['requestBody']
        >,
  ) {
    return this.client.json<CustomTable>({
      path: '/tables/create',
      method: 'POST',
      body: JSON.stringify(
        table instanceof Table ? table.requestBody() : table,
      ),
      projectIdNeeded: true,
    });
  }

  public async update<T>(
    name: T extends Table<infer _, infer U> ? U : string,
    body: {
      name?: string;
      fields?: FieldUpdate[];
    },
  ) {
    return this.client.json<CustomTable>({
      path: `/tables/update/${name}`,
      method: 'PATCH',
      body: JSON.stringify(body),
      projectIdNeeded: true,
    });
  }

  public async delete<T>(name: T extends Table<infer _, infer U> ? U : string) {
    return this.client.json({
      path: `/tables/delete/${name}`,
      method: 'DELETE',
      projectIdNeeded: true,
    });
  }

  public async get<T, U extends boolean>(
    name: T extends Table<infer _, infer U> ? U : string,
    one: U = true as U,
    options?: U extends true
      ? {
          match?: T extends Table<infer _, string>
            ? Partial<TInfer<T>>
            : Partial<BaseData>;
        }
      : {
          one: false;
          page?: number;
          limit?: number;
        } & {
          match?: T extends Table<infer _, string>
            ? Partial<TInfer<T>>
            : Partial<BaseData>;
        },
  ) {
    return this.client.json<
      T extends Table<infer _, string>
        ? TInfer<T>
        : U extends true
          ? UnknownData
          : {
              rows: UnknownData[];
              pagination: { records: number; pages: number };
            }
    >({
      path: merge(
        one === true ? `/tables/${name}/row` : `/tables/${name}/rows`,
        options?.match &&
          `?${Object.entries<
            string | number | boolean | string[] | Date | undefined
          >(options.match)
            .map(([k, v]) => `${k}=${v?.toString()}`)
            .join('&')}`,
      ),
      method: 'GET',
      projectIdNeeded: true,
    });
  }

  public async createRow<T>(
    table: T extends Table<infer _, infer U> ? U : string,
    data: T extends Table<infer _, string>
      ? TInfer<T>
      : Omit<UnknownData, keyof BaseData> & Partial<BaseData>,
  ) {
    return this.client.json<Required<typeof data>>({
      path: `/tables/${table}/create`,
      method: 'POST',
      body: JSON.stringify(data),
      projectIdNeeded: true,
    });
  }

  public async updateRow<T>(
    table: T extends Table<infer _, infer U> ? U : string,
    match: T extends Table<infer _, string>
      ? Partial<TInfer<T>>
      : Partial<BaseData>,
    data: T extends Table<infer _, string>
      ? Partial<TInfer<T>>
      : Partial<UnknownData>,
  ) {
    return this.client.json<Required<typeof data>>({
      path: merge(
        `/tables/${table}/update?`,
        Object.entries<string | number | boolean | string[] | Date | undefined>(
          match,
        )
          .map(([k, v]) => `${k}=${v?.toString()}`)
          .join('&'),
      ),
      method: 'PATCH',
      body: JSON.stringify(data),
      projectIdNeeded: true,
    });
  }

  public async deleteRow<T>(
    table: T extends Table<infer _, infer U> ? U : string,
    match: T extends Table<infer _, string>
      ? Partial<TInfer<T>>
      : Partial<BaseData>,
  ) {
    return this.client.json({
      path: merge(
        `/tables/${table}/delete?`,
        Object.entries<string | number | boolean | string[] | Date | undefined>(
          match,
        )
          .map(([k, v]) => `${k}=${v?.toString()}`)
          .join('&'),
      ),
      method: 'DELETE',
      projectIdNeeded: true,
    });
  }
}
