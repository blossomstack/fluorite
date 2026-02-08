
import Foundation
/// A notification message (adjacently tagged: {type: "...", value: ...})
public enum Message: Codable, Equatable, Sendable {
    case plainText(String)
    case userNotification(UserNotification)
    case orderNotification(OrderNotification)
    case systemAlert(SystemAlert)

    enum CodingKeys: String, CodingKey {
        case type
        case value
    }

    public init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        let type = try container.decode(String.self, forKey: .type)
        switch type {
        case "PlainText":
            self = .plainText(try container.decode(String.self, forKey: .value))
        case "UserNotification":
            self = .userNotification(try container.decode(UserNotification.self, forKey: .value))
        case "OrderNotification":
            self = .orderNotification(try container.decode(OrderNotification.self, forKey: .value))
        case "SystemAlert":
            self = .systemAlert(try container.decode(SystemAlert.self, forKey: .value))
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
        case .plainText(let value):
            try container.encode("PlainText", forKey: .type)
            try container.encode(value, forKey: .value)
        case .userNotification(let value):
            try container.encode("UserNotification", forKey: .type)
            try container.encode(value, forKey: .value)
        case .orderNotification(let value):
            try container.encode("OrderNotification", forKey: .type)
            try container.encode(value, forKey: .value)
        case .systemAlert(let value):
            try container.encode("SystemAlert", forKey: .type)
            try container.encode(value, forKey: .value)
        }
    }
}