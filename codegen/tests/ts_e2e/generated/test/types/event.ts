
import { User } from './user';
export type Event =
  | { eventType: "UserCreated"; user: User; timestamp: string; }
  | { eventType: "UserUpdated"; user: User; changes: string; }
  | { eventType: "UserDeleted"; userId: number; };