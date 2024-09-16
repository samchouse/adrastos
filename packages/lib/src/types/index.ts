export * from './table';

type Options = Omit<RequestInit, 'body'>;

export interface Request {
  path: string;
  body?: BodyInit;
  options?: Options;
  projectIdNeeded?: boolean;
  method: 'GET' | 'DELETE' | 'POST' | 'PATCH' | 'PUT';
}

export interface User {
  id: string;
  email: string;
  banned: boolean;
  createdAt: Date;
  lastName: string;
  username: string;
  updatedAt?: Date;
  firstName: string;
  verified: boolean;
}

export type AndNullable<T> = {
  [P in keyof T]: undefined extends T[P] ? T[P] | null : T[P];
};
