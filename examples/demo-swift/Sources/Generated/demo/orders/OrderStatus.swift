
/// Possible order statuses
public enum OrderStatus: String, Codable, Equatable, Sendable {
    case pending = "Pending"
    case confirmed = "Confirmed"
    case processing = "Processing"
    case shipped = "Shipped"
    case delivered = "Delivered"
    case cancelled = "Cancelled"
    case refunded = "Refunded"
}