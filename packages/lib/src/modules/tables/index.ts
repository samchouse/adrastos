import { Field, FieldUpdate } from '../../types';
import { BaseModule } from '../util';
import { Table, TFWithModifiers } from './fields';

export { table, TFWithModifiers, TInfer, TField } from './fields';

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

  public async create<T extends Record<string, TFWithModifiers>>(
    table: Table<T>,
  ) {
    return this.client.request<CustomTable>({
      path: '/tables/create',
      method: 'POST',
      body: JSON.stringify(table.requestBody()),
    });
  }

  public async update(
    name: string,
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

  public async delete(name: string) {
    return this.client.request({
      path: `/tables/delete/${name}`,
      method: 'DELETE',
    });
  }
}
