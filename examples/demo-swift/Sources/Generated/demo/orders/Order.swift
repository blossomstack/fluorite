
import Foundation
/// Represents a customer order
public struct Order: Codable, Equatable, Sendable {
    /// Unique order identifier
    public let id: String
    /// Reference to the user who placed the order
    public let userId: String
    /// The user who placed the order (denormalized)
    public let user: User?
    /// Items in this order
    public let items: [OrderItem]
    /// Total order amount as string (for precision)
    public let total: String
    /// Order status
    public let status: OrderStatus
    /// Shipping address (imported from common)
    public let shippingAddress: Address
    /// Billing address (if different from shipping)
    public let billingAddress: Address?
    /// When the order was placed
    public let createdAt: String
    /// Optional tracking number
    public let trackingNumber: String?
}