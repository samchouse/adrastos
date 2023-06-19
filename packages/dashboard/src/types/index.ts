export interface User {
  id: string;
  firstName: string;
  lastName: string;
  email: string;
  username: string;
  version: boolean;
  banned: boolean;
  createdAt: Date;
  updatedAt: Date;
}
