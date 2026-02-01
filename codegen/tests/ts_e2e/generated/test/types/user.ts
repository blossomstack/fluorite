
import { UserStatus } from './userStatus';
export interface User {
  id: number;
  name: string;
  email?: string;
  status: UserStatus;
  metadata: unknown;
}