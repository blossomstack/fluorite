
import Foundation
/// An item within an order
public struct OrderItem: Codable, Equatable, Sendable {
    /// Product identifier
    public let productId: String
    /// Product name
    public let name: String
    /// Quantity ordered
    public let quantity: UInt32
    /// Price per unit as string
    public let unitPrice: String
}