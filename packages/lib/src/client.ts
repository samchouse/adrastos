import { ResponseError } from './errors';
import { AccountsModule, TablesModule } from './modules';
import { Request } from './types';

export { table, TFWithModifiers, TInfer, TField } from './modules';

export class Client {
  #authToken = '';

  public accounts: AccountsModule;
  public tables: TablesModule;

  constructor(
    // eslint-disable-next-line no-unused-vars
    private baseUrl: string,
    private _projectId: string,
  ) {
    this.accounts = new AccountsModule(this);
    this.tables = new TablesModule(this);
  }

  public set authToken(token: string) {
    this.#authToken = token;
  }

  public get hasAuthToken() {
    return !!this.#authToken;
  }

  public buildUrl(path: string) {
    return `${this.baseUrl.replace(/\/$/, '')}${path}`;
  }

  async request<T>({ method, path, body, options }: Request): Promise<T> {
    const res = await fetch(this.buildUrl(path), {
      body,
      method,
      ...options,
      headers: {
        'Content-Type': 'application/json',
        Authorization: `Bearer ${this.#authToken}`,
        ...options?.headers,
      },
    });

    if (!res.ok) throw new ResponseError('Something went wrong');
    return res.json() as Promise<T>;
  }
}
