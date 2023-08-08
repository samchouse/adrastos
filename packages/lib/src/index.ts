type Request = OtherRequest | PostRequest;

interface BaseRequest {
  path: string;
  options?: RequestInit;
}

interface OtherRequest extends BaseRequest {
  method?: 'GET' | 'PATCH' | 'PUT' | 'DELETE';
}

interface PostRequest extends BaseRequest {
  method: 'POST';
  body: BodyInit;
  options?: Omit<RequestInit, 'body'>;
}

export class ResponseError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'ResponseError';
  }
}

export class Client {
  public accounts: Accounts;

  // eslint-disable-next-line no-unused-vars
  constructor(private baseUrl: string, private _projectId: string) {
    this.accounts = new Accounts(this);
  }

  async request({ path, method, options }: Request) {
    const res = await fetch(`${this.baseUrl}${path}`, {
      method,
      ...options
    });

    if (!res.ok) {
      throw new ResponseError('Something went wrong');
    }

    return res.json();
  }
}

class Accounts {
  // eslint-disable-next-line no-unused-vars
  constructor(private client: Client) {}

  public async create(body: {
    firstName: string;
    lastName: string;
    email: string;
    username: string;
    password: string;
  }) {
    this.client.request({
      path: '/accounts',
      method: 'POST',
      body: JSON.stringify(body)
    });
  }
}
