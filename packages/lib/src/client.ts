import { ResponseError } from './errors';
import { AccountsModule, TablesModule } from './modules';
import { Request } from './types';

export class Client {
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

  public buildUrl(path: string) {
    return new URL(path, this.baseUrl);
  }

  async request<T>({ path, method, options }: Request): Promise<T> {
    const res = await fetch(this.buildUrl(path), {
      method,
      ...options,
    });

    if (!res.ok) throw new ResponseError('Something went wrong');

    return res.json() as Promise<T>;
  }
}
