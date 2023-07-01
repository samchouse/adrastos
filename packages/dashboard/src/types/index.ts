export interface User {
  id: string;
  firstName: string;
  lastName: string;
  email: string;
  username: string;
  verified: boolean;
  banned: boolean;
  createdAt: Date;
  updatedAt: Date;
}
