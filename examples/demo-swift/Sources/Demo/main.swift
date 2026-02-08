import Foundation
import FluoriteRuntime

// Import generated types - these are in the same module so no explicit import needed
// The generated files are included in Sources/Demo via symlink or direct inclusion

// MARK: - Sample Data Creators

func createSampleAddress() -> Address {
    Address(
        street1: "123 Main St",
        street2: "Apt 4B",
        city: "New York",
        state: "NY",
        postalCode: "10001",
        country: "US"
    )
}

func createSamplePagination() -> Pagination {
    Pagination(
        page: 1,
        perPage: 20,
        totalItems: 156,
        totalPages: 8
    )
}

func createSampleUser() -> User {
    User(
        id: "user-001",
        firstName: "John",
        lastName: "Doe",
        email: "john.doe@example.com",
        age: 30,
        status: .active,
        gender: .male,
        active: true,
        homeAddress: createSampleAddress(),
        createdAt: "2024-01-15T10:30:00Z",
        info: ["hobbies": ["reading", "coding"], "score": 95.5]
    )
}

func createSampleUserMinimal() -> User {
    User(
        id: "user-002",
        firstName: "Jane",
        lastName: "Smith",
        email: "jane.smith@example.com",
        age: nil,
        status: .pending,
        gender: .female,
        active: false,
        homeAddress: nil,
        createdAt: "2024-02-20T14:00:00Z",
        info: nil
    )
}

func createSampleUserEventCreated() -> UserEvent {
    .created(createSampleUser())
}

func createSampleUserEventDeleted() -> UserEvent {
    .deleted
}

func createSampleUserEventStatusChanged() -> UserEvent {
    .statusChanged(UserStatusChange(
        userId: "user-001",
        oldStatus: .pending,
        newStatus: .active,
        changedAt: "2024-01-16T08:00:00Z"
    ))
}

func createSampleOrder() -> Order {
    Order(
        id: "order-001",
        userId: "user-001",
        user: createSampleUser(),
        items: [
            OrderItem(productId: "prod-001", name: "Laptop", quantity: 1, unitPrice: "999.99"),
            OrderItem(productId: "prod-002", name: "Mouse", quantity: 2, unitPrice: "29.99")
        ],
        total: "1059.97",
        status: .confirmed,
        shippingAddress: createSampleAddress(),
        billingAddress: nil,
        createdAt: "2024-01-20T09:00:00Z",
        trackingNumber: "1Z999AA10123456784"
    )
}

func createSampleOrderEventCreated() -> OrderEvent {
    .created(createSampleOrder())
}

func createSampleOrderEventCancelled() -> OrderEvent {
    .cancelled(OrderCancellation(
        orderId: "order-001",
        reason: "Customer requested cancellation",
        refundAmount: "1059.97",
        cancelledAt: "2024-01-21T15:30:00Z"
    ))
}

func createSampleMessagePlainText() -> Message {
    .plainText("Hello, this is a plain text message!")
}

func createSampleMessageSystemAlert() -> Message {
    .systemAlert(SystemAlert(
        title: "Scheduled Maintenance",
        body: "The system will be down for maintenance on Sunday.",
        severity: .warning,
        expiresAt: "2024-01-28T00:00:00Z"
    ))
}

func createSampleQueuedNotification() -> QueuedNotification {
    QueuedNotification(
        id: "notif-001",
        message: .userNotification(UserNotification(
            title: "Welcome!",
            body: "Thank you for signing up.",
            userId: "user-001",
            actionUrl: "https://example.com/welcome"
        )),
        recipientId: "user-001",
        status: .delivered,
        createdAt: "2024-01-15T10:31:00Z",
        sentAt: "2024-01-15T10:31:05Z",
        deliveredAt: "2024-01-15T10:31:10Z",
        readAt: nil
    )
}

// MARK: - JSON Helpers

let encoder: JSONEncoder = {
    let encoder = JSONEncoder()
    encoder.outputFormatting = [.prettyPrinted, .sortedKeys]
    return encoder
}()

let decoder = JSONDecoder()

func toJSON<T: Encodable>(_ value: T) -> String {
    guard let data = try? encoder.encode(value),
          let json = String(data: data, encoding: .utf8) else {
        return "{}"
    }
    return json
}

func fromJSON<T: Decodable>(_ json: String, as type: T.Type) -> T? {
    guard let data = json.data(using: .utf8) else { return nil }
    return try? decoder.decode(type, from: data)
}

// MARK: - File Operations

func writeFixtures(to directory: URL) throws {
    try FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true)

    // Common types
    try toJSON(createSampleAddress()).write(to: directory.appendingPathComponent("address.json"), atomically: true, encoding: .utf8)
    try toJSON(createSamplePagination()).write(to: directory.appendingPathComponent("pagination.json"), atomically: true, encoding: .utf8)

    // User types
    try toJSON(createSampleUser()).write(to: directory.appendingPathComponent("user.json"), atomically: true, encoding: .utf8)
    try toJSON(createSampleUserMinimal()).write(to: directory.appendingPathComponent("user_minimal.json"), atomically: true, encoding: .utf8)
    try toJSON(createSampleUserEventCreated()).write(to: directory.appendingPathComponent("user_event_created.json"), atomically: true, encoding: .utf8)
    try toJSON(createSampleUserEventDeleted()).write(to: directory.appendingPathComponent("user_event_deleted.json"), atomically: true, encoding: .utf8)
    try toJSON(createSampleUserEventStatusChanged()).write(to: directory.appendingPathComponent("user_event_status_changed.json"), atomically: true, encoding: .utf8)

    // Order types
    try toJSON(createSampleOrder()).write(to: directory.appendingPathComponent("order.json"), atomically: true, encoding: .utf8)
    try toJSON(createSampleOrderEventCreated()).write(to: directory.appendingPathComponent("order_event_created.json"), atomically: true, encoding: .utf8)
    try toJSON(createSampleOrderEventCancelled()).write(to: directory.appendingPathComponent("order_event_cancelled.json"), atomically: true, encoding: .utf8)

    // Notification types
    try toJSON(createSampleMessagePlainText()).write(to: directory.appendingPathComponent("message_plain.json"), atomically: true, encoding: .utf8)
    try toJSON(createSampleMessageSystemAlert()).write(to: directory.appendingPathComponent("message_system_alert.json"), atomically: true, encoding: .utf8)
    try toJSON(createSampleQueuedNotification()).write(to: directory.appendingPathComponent("queued_notification.json"), atomically: true, encoding: .utf8)

    print("Fixtures written to \(directory.path)")
}

func readAndVerifyFixtures(from directory: URL) throws {
    let files = try FileManager.default.contentsOfDirectory(at: directory, includingPropertiesForKeys: nil)
        .filter { $0.pathExtension == "json" }

    print("\n=== Reading fixtures from \(directory.path) ===\n")

    for file in files.sorted(by: { $0.lastPathComponent < $1.lastPathComponent }) {
        let content = try String(contentsOf: file, encoding: .utf8)
        let name = file.deletingPathExtension().lastPathComponent

        print("--- \(name).json ---")

        // Try to decode based on filename pattern
        var success = false

        if name.contains("user_event") {
            if let event = fromJSON(content, as: UserEvent.self) {
                print("✓ Decoded as UserEvent: \(event)")
                success = true
            }
        } else if name.contains("user") && !name.contains("notification") {
            if let user = fromJSON(content, as: User.self) {
                print("✓ Decoded as User: \(user.firstName) \(user.lastName)")
                success = true
            }
        } else if name.contains("order_event") {
            if let event = fromJSON(content, as: OrderEvent.self) {
                print("✓ Decoded as OrderEvent: \(event)")
                success = true
            }
        } else if name.contains("order") && !name.contains("notification") {
            if let order = fromJSON(content, as: Order.self) {
                print("✓ Decoded as Order: \(order.id)")
                success = true
            }
        } else if name.contains("message") {
            if let msg = fromJSON(content, as: Message.self) {
                print("✓ Decoded as Message: \(msg)")
                success = true
            }
        } else if name.contains("address") {
            if let addr = fromJSON(content, as: Address.self) {
                print("✓ Decoded as Address: \(addr.city), \(addr.country)")
                success = true
            }
        } else if name.contains("pagination") {
            if let page = fromJSON(content, as: Pagination.self) {
                print("✓ Decoded as Pagination: page \(page.page) of \(page.totalPages)")
                success = true
            }
        } else if name.contains("notification") {
            if let notif = fromJSON(content, as: QueuedNotification.self) {
                print("✓ Decoded as QueuedNotification: \(notif.id)")
                success = true
            }
        }

        if !success {
            print("? Unknown file type, raw content:")
            print(content.prefix(200))
        }
        print()
    }
}

func runDemo() {
    print("=== Fluorite Swift Multi-Package Demo ===\n")

    // Demo cross-package types
    print("--- Common Types ---")
    let addr = createSampleAddress()
    print("Address: \(addr.city), \(addr.state), \(addr.country)\n")

    // Demo User with Address from common
    print("--- User Package (imports common.Address) ---")
    let user = createSampleUser()
    print("User: \(user.firstName) \(user.lastName)")
    if let home = user.homeAddress {
        print("  Home: \(home.city), \(home.country)")
    }
    print()

    // Demo Order with User and Address
    print("--- Order Package (imports common.Address, users.User) ---")
    let order = createSampleOrder()
    print("Order: \(order.id) - \(order.items.count) items, total: \(order.total)")
    if let orderUser = order.user {
        print("  Placed by: \(orderUser.firstName) \(orderUser.lastName)")
    }
    print("  Ship to: \(order.shippingAddress.city), \(order.shippingAddress.country)")
    print()

    // Demo Notifications with adjacently tagged union
    print("--- Notification Package (adjacently tagged union) ---")
    let msg = createSampleMessageSystemAlert()
    if case .systemAlert(let alert) = msg {
        print("System Alert: \(alert.title) (severity: \(alert.severity))")
    }
    print()

    // Demo JSON serialization
    print("=== JSON Serialization Examples ===\n")

    print("User JSON:")
    print(toJSON(user))
    print()

    print("UserEvent::StatusChanged JSON:")
    let event = createSampleUserEventStatusChanged()
    print(toJSON(event))
    print()

    print("Message::SystemAlert JSON:")
    print(toJSON(msg))
    print()
}

// MARK: - Main

let args = CommandLine.arguments

if args.count < 2 {
    runDemo()
} else {
    let command = args[1]
    let dir = args.count > 2 ? args[2] : "./fixtures"
    let directory = URL(fileURLWithPath: dir)

    do {
        switch command {
        case "--write":
            try writeFixtures(to: directory)
        case "--read":
            try readAndVerifyFixtures(from: directory)
        default:
            print("Usage: Demo [command]")
            print("  (no args)      Run demo showing type examples")
            print("  --write <dir>  Write sample JSON files to directory")
            print("  --read <dir>   Read and verify JSON files from directory")
        }
    } catch {
        print("Error: \(error)")
        exit(1)
    }
}
