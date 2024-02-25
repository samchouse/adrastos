export * from './table';

type Options = Omit<RequestInit, 'body'>;

export interface Request {
  method: 'GET' | 'DELETE' | 'POST' | 'PATCH' | 'PUT';
  path: string;
  body?: BodyInit;
  options?: Options;
  projectIdNeeded?: boolean;
}

export interface User {
  id: string;
  firstName: string;
  lastName: string;
  email: string;
  username: string;
  verified: boolean;
  banned: boolean;
  createdAt: Date;
  updatedAt?: Date;
}

export type AndNullable<T> = {
  [P in keyof T]: undefined extends T[P] ? T[P] | null : T[P];
};
