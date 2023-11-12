import { ResponseError } from './errors';
import { AccountsModule, ConfigModule, TablesModule } from './modules';
import { Request } from './types';

export { table, TFWithModifiers, TInfer, TField, CustomTable } from './modules';

export class Client {
  #authToken?: string;

  public accounts: AccountsModule;
  public tables: TablesModule;
  public config: ConfigModule;

  constructor(
    private baseUrl: string,
    private _projectId: string,
  ) {
    this.accounts = new AccountsModule(this);
    this.tables = new TablesModule(this);
    this.config = new ConfigModule(this);
  }

  public set authToken(token: string | undefined) {
    this.#authToken = token;
  }

  public get hasAuthToken() {
    return !!this.#authToken;
  }

  public buildUrl(path: string) {
    return `${this.baseUrl.replace(/\/$/, '')}/api${path}`;
  }

  async request<T>({ method, path, body, options }: Request): Promise<T> {
    const res = await fetch(this.buildUrl(path), {
      body,
      method,
      ...options,
      headers: {
        'Content-Type': 'application/json',
        ...options?.headers,
        ...(this.#authToken && { Authorization: `Bearer ${this.#authToken}` }),
      },
    });

    if (!res.ok) throw new ResponseError('Something went wrong');
    return res.json() as Promise<T>;
  }
}
