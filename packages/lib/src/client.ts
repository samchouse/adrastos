import { ResponseError } from './errors';
import { AccountsModule, ConfigModule, TablesModule } from './modules';
import type { Request } from './types';

export {
  CustomTable,
  TFWithModifiers,
  TField,
  TInfer,
  Table,
  table,
} from './modules';

export class Client {
  #authToken?: string;
  public projectId: string | undefined;
  private onlyUseProjectIdIfNeeded = false;

  public accounts: AccountsModule;
  public tables: TablesModule;
  public config: ConfigModule;

  constructor(private baseUrl: string) {
    this.accounts = new AccountsModule(this);
    this.tables = new TablesModule(this);
    this.config = new ConfigModule(this);
  }

  public setProjectId(id: string, onlyIfNeeded = false) {
    this.projectId = id;
    this.onlyUseProjectIdIfNeeded = onlyIfNeeded;
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

  async request({ method, path, body, options, projectIdNeeded }: Request) {
    return await fetch(this.buildUrl(path), {
      body,
      method,
      ...options,
      headers: {
        ...options?.headers,
        ...(this.#authToken && { Authorization: `Bearer ${this.#authToken}` }),
        ...(!(body instanceof FormData) && {
          'Content-Type': 'application/json',
        }),
        ...((this.onlyUseProjectIdIfNeeded
          ? projectIdNeeded && this.projectId
          : this.projectId) && { 'X-Project-Id': this.projectId }),
      },
    });
  }

  async json<T = null>(options: Request) {
    const res = await this.request(options);

    if (!res.ok)
      throw new ResponseError(
        'Something went wrong',
        (await res.json()) as ResponseError['details'],
      );
    return res.json() as T;
  }
}
