import { ResponseError } from './errors';
import { AccountsModule, ConfigModule, TablesModule } from './modules';
import { Request } from './types';

export {
  table,
  Table,
  TFWithModifiers,
  TInfer,
  TField,
  CustomTable,
} from './modules';

export class Client {
  #authToken?: string;
  private projectId: string | undefined;
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

  async request<T = null>({
    method,
    path,
    body,
    options,
    projectIdNeeded,
  }: Request) {
    const res = await fetch(this.buildUrl(path), {
      body,
      method,
      ...options,
      headers: {
        'Content-Type': 'application/json',
        ...options?.headers,
        ...(this.#authToken && { Authorization: `Bearer ${this.#authToken}` }),
        ...((this.onlyUseProjectIdIfNeeded
          ? projectIdNeeded && this.projectId
          : this.projectId) && { 'X-Project-Id': this.projectId }),
      },
    });

    if (!res.ok)
      throw new ResponseError(
        'Something went wrong',
        (await res.json()) as ResponseError['details'],
      );
    return res.json() as T;
  }
}
