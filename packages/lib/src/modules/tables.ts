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
} from '../types';
import { BaseModule } from './util';

export class TablesModule extends BaseModule {
  public create(body: {
    name: string;
    stringFields?: StringField[];
    numberFields?: NumberField[];
    booleanFields?: BooleanField[];
    dateFields?: DateField[];
    emailFields?: EmailField[];
    urlFields?: UrlField[];
    selectFields?: SelectField[];
    relationFields?: RelationField[];
  }) {
    return this.client.request({
      path: '/tables/create',
      method: 'POST',
      body: JSON.stringify(body),
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
