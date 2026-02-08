
/// Notification delivery status
public enum DeliveryStatus: String, Codable, Equatable, Sendable {
    case pending = "Pending"
    case sent = "Sent"
    case delivered = "Delivered"
    case failed = "Failed"
    case read = "Read"
}