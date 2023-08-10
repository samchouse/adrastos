export * from './table';

type Options = Omit<RequestInit, 'body'>;

interface BaseRequest {
  path: string;
  options?: RequestInit;
}

interface BasicRequest extends BaseRequest {
  method?: 'GET' | 'DELETE';
  options?: Options;
}

interface BodyRequest extends BaseRequest {
  method: 'POST' | 'PATCH' | 'PUT';
  body: BodyInit;
  options?: Options;
}

export type Request = BasicRequest | BodyRequest;
