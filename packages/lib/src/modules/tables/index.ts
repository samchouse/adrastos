import {
  BooleanField,
  DateField,
  EmailField,
  NumberField,
  RelationField,
  SelectField,
  StringField,
  UpdateField,
  UrlField,
} from '../../types';
import { Table, TFWithModifiers } from '../tables/fields';
import { BaseModule } from '../util';

export class TablesModule extends BaseModule {
  public create<T extends Record<string, TFWithModifiers>>(table: Table<T>) {
    return this.client.request({
      path: '/tables/create',
      method: 'POST',
      body: JSON.stringify(table.requestBody()),
    });
  }

  public update(
    name: string,
    body: {
      stringFields?: UpdateField<StringField>[];
      numberFields?: UpdateField<NumberField>[];
      booleanFields?: UpdateField<BooleanField>[];
      dateFields?: UpdateField<DateField>[];
      emailFields?: UpdateField<EmailField>[];
      urlFields?: UpdateField<UrlField>[];
      selectFields?: UpdateField<SelectField>[];
      relationFields?: UpdateField<RelationField>[];
    },
  ) {
    return this.client.request({
      path: `/tables/update/${name}`,
      method: 'PATCH',
      body: JSON.stringify(body),
    });
  }

  public delete(name: string) {
    return this.client.request({
      path: `/tables/delete/${name}`,
      method: 'DELETE',
    });
  }
}
