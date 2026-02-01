// This file uses the generated types to verify they work correctly

import {
  User,
  UserStatus,
  Event,
  UserList,
  UserMap
} from '../generated/test/types';

// Test interface usage
const user: User = {
  id: 1,
  name: "John Doe",
  email: "john@example.com",
  status: UserStatus.Active,
  metadata: { role: "admin" }
};

// Test optional field
const userWithoutEmail: User = {
  id: 2,
  name: "Jane Doe",
  status: UserStatus.Pending,
  metadata: null
};

// Test enum
function getUserStatusLabel(status: UserStatus): string {
  switch (status) {
    case UserStatus.Active:
      return "Active User";
    case UserStatus.Inactive:
      return "Inactive User";
    case UserStatus.Pending:
      return "Pending Approval";
  }
}

// Test discriminated union with type narrowing
function handleEvent(event: Event): string {
  switch (event.eventType) {
    case "UserCreated":
      return `User ${event.user.name} created at ${event.timestamp}`;
    case "UserUpdated":
      return `User ${event.user.name} updated: ${event.changes}`;
    case "UserDeleted":
      return `User ${event.userId} deleted`;
  }
}

// Test type aliases
const users: UserList = [user, userWithoutEmail];
const userById: UserMap = {
  "1": user,
  "2": userWithoutEmail
};

// Verify everything compiles
console.log("User:", user);
console.log("Status label:", getUserStatusLabel(user.status));
console.log("Users count:", users.length);
console.log("User map keys:", Object.keys(userById));
