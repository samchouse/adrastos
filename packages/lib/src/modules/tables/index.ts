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

export class TablesModule extends BaseModule {
  public async list() {
    return this.client.request<CustomTable[]>({
      path: '/tables/list',
      method: 'GET',
    });
  }

  public async create<T>(
    table: T extends Table<infer _, string>
      ? T
      : ReturnType<
          Table<Record<string, TFWithModifiers>, string>['requestBody']
        >,
  ) {
    return this.client.request<CustomTable>({
      path: '/tables/create',
      method: 'POST',
      body: JSON.stringify(
        table instanceof Table ? table.requestBody() : table,
      ),
    });
  }

  public async update<T>(
    name: T extends Table<infer _, infer U> ? U : string,
    body: {
      name?: string;
      fields?: FieldUpdate[];
    },
  ) {
    return this.client.request<CustomTable>({
      path: `/tables/update/${name}`,
      method: 'PATCH',
      body: JSON.stringify(body),
    });
  }

  public async delete<T>(name: T extends Table<infer _, infer U> ? U : string) {
    return this.client.request({
      path: `/tables/delete/${name}`,
      method: 'DELETE',
    });
  }

  public async get<T>(
    name: T extends Table<infer _, infer U> ? U : string,
    options?: (
      | { one: true }
      | {
          one: false;
          page?: number;
          limit?: number;
        }
    ) & {
      match?: T extends Table<infer _, string>
        ? Partial<TInfer<T>>
        : Record<string, string>;
    },
  ) {
    return this.client.request<CustomTable>({
      path: merge(
        options?.one === true ? `/tables/${name}/row` : `/tables/${name}/rows`,
        options?.match &&
          `?${Object.entries<
            string | number | boolean | string[] | Date | undefined
          >(options.match)
            .map(([k, v]) => `${k}=${v?.toString()}`)
            .join('&')}`,
      ),
      method: 'GET',
    });
  }

  public async createRow<T>(
    table: T extends Table<infer _, infer U> ? U : string,
    data: T extends Table<infer _, string>
      ? TInfer<T>
      : {
          id?: string;
          createdAt?: Date;
          updatedAt?: Date;
          [key: string]: unknown;
        },
  ) {
    return this.client.request<CustomTable>({
      path: `/tables/${table}/create`,
      method: 'POST',
      body: JSON.stringify(data),
    });
  }

  public async updateRow<T>(
    table: T extends Table<infer _, infer U> ? U : string,
    match: T extends Table<infer _, string>
      ? Partial<TInfer<T>>
      : Record<string, string>,
    data: T extends Table<infer _, string>
      ? TInfer<T>
      : {
          id?: string;
          createdAt?: Date;
          updatedAt?: Date;
          [key: string]: unknown;
        },
  ) {
    return this.client.request<typeof data>({
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
    });
  }

  public async deleteRow<T>(
    table: T extends Table<infer _, infer U> ? U : string,
    match: T extends Table<infer _, string>
      ? Partial<TInfer<T>>
      : Record<string, string>,
  ) {
    return this.client.request({
      path: merge(
        `/tables/${table}/update?`,
        Object.entries<string | number | boolean | string[] | Date | undefined>(
          match,
        )
          .map(([k, v]) => `${k}=${v?.toString()}`)
          .join('&'),
      ),
      method: 'DELETE',
    });
  }
}
