import type { Field, FieldCrud } from '../../types';
import { merge } from '../../util';
import { BaseModule } from '../util';
/* eslint-disable @stylistic/max-len */
import {
  type BaseData,
  type Row,
  TFWithModifiers,
  type TInfer,
  Table,
} from './fields';

export { Row, TFWithModifiers, TField, TInfer, Table, table } from './fields';

export interface CustomTable {
  id: string;
  name: string;
  fields: Field[];
  createdAt: string;
  updatedAt: string;
  permissions: {
    view: string | null;
    create: string | null;
    update: string | null;
    delete: string | null;
  };
}

type TableName<T> = T extends Table<infer U> ? U : string;
type OptionalTableData<T extends Row | Table> = Partial<
  T extends Table ? TInfer<T> : T
>;

export class TablesModule extends BaseModule {
  public async list() {
    return this.client.json<CustomTable[]>({
      path: '/tables/list',
      method: 'GET',
      projectIdNeeded: true,
    });
  }

  public async create<T>(
    table: T extends Table ? T : ReturnType<Table['requestBody']>,
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
    name: TableName<T>,
    body: {
      name?: string;
      fields?: FieldCrud[];
      permissions?: {
        view: string | null;
        create: string | null;
        update: string | null;
        delete: string | null;
      };
    },
  ) {
    return this.client.json<CustomTable>({
      path: `/tables/update/${name}`,
      method: 'PATCH',
      body: JSON.stringify(body),
      projectIdNeeded: true,
    });
  }

  public async delete<T>(name: TableName<T>) {
    return this.client.json({
      path: `/tables/delete/${name}`,
      method: 'DELETE',
      projectIdNeeded: true,
    });
  }

  public async getOne<T extends Row | Table = Row>(
    name: TableName<T>,
    id: string,
  ) {
    return this.client.json<T extends Table ? TInfer<T> : T>({
      path: `/tables/${name}/rows/${id}`,
      method: 'GET',
      projectIdNeeded: true,
    });
  }

  public async get<T extends Row | Table>(
    name: TableName<T>,
    options: {
      page?: number;
      limit?: number;
      match?: OptionalTableData<T>;
    },
  ) {
    return this.client.json<{
      rows: (T extends Table ? TInfer<T> : T)[];
      pagination: {
        records: number;
        pages: number;
      };
    }>({
      path: merge(
        `/tables/${name}/rows`,
        options.match &&
          `?${Object.entries<
            string | number | boolean | string[] | Date | undefined
          >(options.match)
            .map(([k, v]) => `${k}=${v?.toString() ?? ''}`)
            .join('&')}`,
      ),
      method: 'GET',
      projectIdNeeded: true,
    });
  }

  public async createRow<T extends Row | Table = Row>(
    table: TableName<T>,
    data: T extends Table
      ? TInfer<T>
      : Omit<T, keyof BaseData> & Partial<BaseData>,
  ) {
    return this.client.json<Required<typeof data>>({
      path: `/tables/${table}/create`,
      method: 'POST',
      body: JSON.stringify(data),
      projectIdNeeded: true,
    });
  }

  public async updateRow<T extends Row | Table = Row>(
    table: TableName<T>,
    match: OptionalTableData<T>,
    data: OptionalTableData<T>,
  ) {
    return this.client.json<Required<typeof data>>({
      path: merge(
        `/tables/${table}/update?`,
        Object.entries<string | number | boolean | string[] | Date | undefined>(
          match,
        )
          .map(([k, v]) => `${k}=${v?.toString() ?? ''}`)
          .join('&'),
      ),
      method: 'PATCH',
      body: JSON.stringify(data),
      projectIdNeeded: true,
    });
  }

  public async deleteRow<T extends Row | Table = Row>(
    table: TableName<T>,
    match: OptionalTableData<T>,
  ) {
    return this.client.json({
      path: merge(
        `/tables/${table}/delete?`,
        Object.entries<string | number | boolean | string[] | Date | undefined>(
          match,
        )
          .map(([k, v]) => `${k}=${v?.toString() ?? ''}`)
          .join('&'),
      ),
      method: 'DELETE',
      projectIdNeeded: true,
    });
  }
}

const a = new TablesModule();

const b = a.get('asds');
void b.then((d) => d.rows[0].asd);
