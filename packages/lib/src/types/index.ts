export * from './table';

type Options = Omit<RequestInit, 'body'>;

export interface Request {
  method: 'GET' | 'DELETE' | 'POST' | 'PATCH' | 'PUT';
  path: string;
  body?: BodyInit;
  options?: Options;
}

export interface User {
  id: string;
  firstName: string;
  lastName: string;
  email: string;
  username: string;
  verified: boolean;
  banned: boolean;
  createdAt: string;
  updatedAt: string;
}
