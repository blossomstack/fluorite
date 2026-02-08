
import Foundation
/// Event types for order lifecycle
public enum OrderEvent: Codable, Equatable, Sendable {
    case created(Order)
    case updated(Order)
    case statusChanged(OrderStatusChange)
    case cancelled(OrderCancellation)

    enum CodingKeys: String, CodingKey {
        case type
        case value
    }

    public init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        let type = try container.decode(String.self, forKey: .type)
        switch type {
        case "Created":
            self = .created(try container.decode(Order.self, forKey: .value))
        case "Updated":
            self = .updated(try container.decode(Order.self, forKey: .value))
        case "StatusChanged":
            self = .statusChanged(try container.decode(OrderStatusChange.self, forKey: .value))
        case "Cancelled":
            self = .cancelled(try container.decode(OrderCancellation.self, forKey: .value))
        default:
            throw DecodingError.dataCorrupted(
                DecodingError.Context(
                    codingPath: decoder.codingPath,
                    debugDescription: "Unknown type: \(type)"
                )
            )
        }
    }

    public func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)
        switch self {
        case .created(let value):
            try container.encode("Created", forKey: .type)
            try container.encode(value, forKey: .value)
        case .updated(let value):
            try container.encode("Updated", forKey: .type)
            try container.encode(value, forKey: .value)
        case .statusChanged(let value):
            try container.encode("StatusChanged", forKey: .type)
            try container.encode(value, forKey: .value)
        case .cancelled(let value):
            try container.encode("Cancelled", forKey: .type)
            try container.encode(value, forKey: .value)
        }
    }
}