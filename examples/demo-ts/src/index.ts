import * as fs from "fs";
import * as path from "path";

// Import generated types from multiple packages
import { Address, ApiResponse, Pagination } from "../generated/demo/common";
import {
  User,
  Gender,
  UserStatus,
  UserEvent,
  UserStatusChange,
} from "../generated/demo/users";
import {
  Order,
  OrderStatus,
  OrderEvent,
  OrderStatusChange,
  OrderCancellation,
} from "../generated/demo/orders";
import {
  Message,
  UserNotification,
  OrderNotification,
  SystemAlert,
  AlertSeverity,
  DeliveryStatus,
  QueuedNotification,
} from "../generated/demo/notifications";

// Serialization helpers
function serializeToJson(data: unknown): string {
  return JSON.stringify(data, null, 2);
}

// Sample data creators - Common
function createSampleAddress(): Address {
  return {
    street1: "123 Main St",
    street2: "Apt 4B",
    city: "New York",
    state: "NY",
    postalCode: "10001",
    country: "US",
  };
}

function createSampleApiResponseSuccess(): ApiResponse {
  return {
    success: true,
    data: { users: 42, orders: 15 },
    requestId: "req-12345",
  };
}

function createSampleApiResponseError(): ApiResponse {
  return {
    success: false,
    errorMessage: "User not found",
    errorCode: "USER_NOT_FOUND",
    requestId: "req-12346",
  };
}

function createSamplePagination(): Pagination {
  return {
    page: 1,
    perPage: 20,
    totalItems: 156,
    totalPages: 8,
  };
}

// Sample data creators - Users
function createSampleUser(): User {
  return {
    id: "user-001",
    firstName: "John",
    lastName: "Doe",
    email: "john.doe@example.com",
    age: 30,
    status: UserStatus.Active,
    gender: Gender.Male,
    active: true,
    homeAddress: createSampleAddress(),
    createdAt: "2024-01-15T10:30:00Z",
    info: {
      hobbies: ["reading", "coding"],
      score: 95.5,
    },
  };
}

function createSampleUserEventCreated(): UserEvent {
  return {
    type: "Created",
    value: createSampleUser(),
  };
}

function createSampleUserEventStatusChanged(): UserEvent {
  return {
    type: "StatusChanged",
    value: {
      userId: "user-001",
      oldStatus: UserStatus.Pending,
      newStatus: UserStatus.Active,
      changedAt: "2024-01-16T08:00:00Z",
    } as UserStatusChange,
  };
}

function createSampleUserMinimal(): User {
  return {
    id: "user-002",
    firstName: "Jane",
    lastName: "Smith",
    email: "jane.smith@example.com",
    status: UserStatus.Pending,
    gender: Gender.Female,
    active: false,
    createdAt: "2024-02-20T14:00:00Z",
  };
}

function createSampleUserEventDeleted(): UserEvent {
  return {
    type: "Deleted",
  };
}

// Sample data creators - Orders
function createSampleOrder(): Order {
  return {
    id: "order-001",
    userId: "user-001",
    user: createSampleUser(),
    items: [
      {
        productId: "prod-001",
        name: "Laptop",
        quantity: 1,
        unitPrice: "999.99",
      },
      {
        productId: "prod-002",
        name: "Mouse",
        quantity: 2,
        unitPrice: "29.99",
      },
    ],
    total: "1059.97",
    status: OrderStatus.Confirmed,
    shippingAddress: createSampleAddress(),
    createdAt: "2024-01-20T09:00:00Z",
    trackingNumber: "1Z999AA10123456784",
  };
}

function createSampleOrderEventCreated(): OrderEvent {
  return {
    type: "Created",
    value: createSampleOrder(),
  };
}

function createSampleOrderEventCancelled(): OrderEvent {
  return {
    type: "Cancelled",
    value: {
      orderId: "order-001",
      reason: "Customer requested cancellation",
      refundAmount: "1059.97",
      cancelledAt: "2024-01-21T15:30:00Z",
    } as OrderCancellation,
  };
}

function createSampleOrderEventStatusChanged(): OrderEvent {
  return {
    type: "StatusChanged",
    value: {
      orderId: "order-001",
      oldStatus: OrderStatus.Pending,
      newStatus: OrderStatus.Confirmed,
      changedAt: "2024-01-20T10:00:00Z",
    } as OrderStatusChange,
  };
}

// Sample data creators - Notifications
function createSampleMessagePlainText(): Message {
  return {
    type: "PlainText",
    value: "Hello, this is a plain text message!",
  };
}

function createSampleMessageUserNotification(): Message {
  return {
    type: "UserNotification",
    value: {
      title: "Welcome!",
      body: "Thank you for signing up.",
      userId: "user-001",
      actionUrl: "https://example.com/welcome",
    } as UserNotification,
  };
}

function createSampleMessageOrderNotification(): Message {
  return {
    type: "OrderNotification",
    value: {
      title: "Order Shipped!",
      body: "Your order is on its way.",
      orderId: "order-001",
      actionUrl: "https://example.com/track/order-001",
    } as OrderNotification,
  };
}

function createSampleMessageSystemAlert(): Message {
  return {
    type: "SystemAlert",
    value: {
      title: "Scheduled Maintenance",
      body: "The system will be down for maintenance on Sunday.",
      severity: AlertSeverity.Warning,
      expiresAt: "2024-01-28T00:00:00Z",
    } as SystemAlert,
  };
}

function createSampleQueuedNotification(): QueuedNotification {
  return {
    id: "notif-001",
    message: createSampleMessageUserNotification(),
    recipientId: "user-001",
    status: DeliveryStatus.Delivered,
    createdAt: "2024-01-15T10:31:00Z",
    sentAt: "2024-01-15T10:31:05Z",
    deliveredAt: "2024-01-15T10:31:10Z",
  };
}

// Write sample data to files
function writeSampleData(outputDir: string): void {
  if (!fs.existsSync(outputDir)) {
    fs.mkdirSync(outputDir, { recursive: true });
  }

  // Write Common samples
  fs.writeFileSync(
    path.join(outputDir, "address.json"),
    serializeToJson(createSampleAddress())
  );
  fs.writeFileSync(
    path.join(outputDir, "api_response_success.json"),
    serializeToJson(createSampleApiResponseSuccess())
  );
  fs.writeFileSync(
    path.join(outputDir, "api_response_error.json"),
    serializeToJson(createSampleApiResponseError())
  );
  fs.writeFileSync(
    path.join(outputDir, "pagination.json"),
    serializeToJson(createSamplePagination())
  );

  // Write User samples
  fs.writeFileSync(
    path.join(outputDir, "user.json"),
    serializeToJson(createSampleUser())
  );
  fs.writeFileSync(
    path.join(outputDir, "user_minimal.json"),
    serializeToJson(createSampleUserMinimal())
  );
  fs.writeFileSync(
    path.join(outputDir, "user_event_created.json"),
    serializeToJson(createSampleUserEventCreated())
  );
  fs.writeFileSync(
    path.join(outputDir, "user_event_status_changed.json"),
    serializeToJson(createSampleUserEventStatusChanged())
  );

  // Write Order samples
  fs.writeFileSync(
    path.join(outputDir, "order.json"),
    serializeToJson(createSampleOrder())
  );
  fs.writeFileSync(
    path.join(outputDir, "order_event_created.json"),
    serializeToJson(createSampleOrderEventCreated())
  );
  fs.writeFileSync(
    path.join(outputDir, "order_event_cancelled.json"),
    serializeToJson(createSampleOrderEventCancelled())
  );
  fs.writeFileSync(
    path.join(outputDir, "order_event_status_changed.json"),
    serializeToJson(createSampleOrderEventStatusChanged())
  );

  // Write Notification samples
  fs.writeFileSync(
    path.join(outputDir, "message_plain.json"),
    serializeToJson(createSampleMessagePlainText())
  );
  fs.writeFileSync(
    path.join(outputDir, "message_user_notification.json"),
    serializeToJson(createSampleMessageUserNotification())
  );
  fs.writeFileSync(
    path.join(outputDir, "message_order_notification.json"),
    serializeToJson(createSampleMessageOrderNotification())
  );
  fs.writeFileSync(
    path.join(outputDir, "message_system_alert.json"),
    serializeToJson(createSampleMessageSystemAlert())
  );
  fs.writeFileSync(
    path.join(outputDir, "queued_notification.json"),
    serializeToJson(createSampleQueuedNotification())
  );

  console.log(`Sample data written to ${outputDir}`);
}

// Read and display JSON files
function readAndDisplay(inputDir: string): void {
  const files = fs.readdirSync(inputDir).filter((f) => f.endsWith(".json"));

  for (const file of files) {
    const filePath = path.join(inputDir, file);
    const content = fs.readFileSync(filePath, "utf-8");
    console.log(`\n=== ${file} ===`);
    console.log(content);
  }
}

// Run demo
function runDemo(): void {
  console.log("=== Fluorite TypeScript Multi-Package Demo ===\n");

  // Demo cross-package types
  console.log("--- Common Types ---");
  const addr = createSampleAddress();
  console.log(`Address: ${addr.city}, ${addr.state}, ${addr.country}\n`);

  // Demo User with Address from common
  console.log("--- User Package (imports common.Address) ---");
  const user = createSampleUser();
  console.log(`User: ${user.firstName} ${user.lastName}`);
  if (user.homeAddress) {
    console.log(`  Home: ${user.homeAddress.city}, ${user.homeAddress.country}`);
  }
  console.log();

  // Demo Order with User and Address
  console.log("--- Order Package (imports common.Address, users.User) ---");
  const order = createSampleOrder();
  console.log(`Order: ${order.id} - ${order.items.length} items, total: ${order.total}`);
  if (order.user) {
    console.log(`  Placed by: ${order.user.firstName} ${order.user.lastName}`);
  }
  console.log(`  Ship to: ${order.shippingAddress.city}, ${order.shippingAddress.country}`);
  console.log();

  // Demo Notifications with adjacently tagged union
  console.log("--- Notification Package (adjacently tagged union) ---");
  const msg = createSampleMessageSystemAlert();
  if (msg.type === "SystemAlert") {
    const alert = msg.value as SystemAlert;
    console.log(`System Alert: ${alert.title} (severity: ${alert.severity})`);
  }
  console.log();

  // Demo JSON serialization
  console.log("=== JSON Serialization Examples ===\n");

  console.log("User JSON:");
  console.log(serializeToJson(user));
  console.log();

  console.log("UserEvent::StatusChanged JSON:");
  const event = createSampleUserEventStatusChanged();
  console.log(serializeToJson(event));
  console.log();

  console.log("Message::SystemAlert JSON:");
  console.log(serializeToJson(msg));
  console.log();
}

// Main CLI
function main(): void {
  const args = process.argv.slice(2);

  if (args.length === 0) {
    runDemo();
    return;
  }

  const command = args[0];
  const dir = args[1] || "./fixtures";

  switch (command) {
    case "--write":
      writeSampleData(dir);
      break;
    case "--read":
      readAndDisplay(dir);
      break;
    default:
      console.log("Usage: ts-node index.ts [command]");
      console.log("  (no args)      Run demo showing type examples");
      console.log("  --write <dir>  Write sample JSON files to directory");
      console.log("  --read <dir>   Read and display JSON files from directory");
      process.exit(0);
  }
}

main();
