
import Foundation
/// Event types for user lifecycle
public enum UserEvent: Codable, Equatable, Sendable {
    case created(User)
    case updated(User)
    case deleted
    case statusChanged(UserStatusChange)

    enum CodingKeys: String, CodingKey {
        case type
        case value
    }

    public init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        let type = try container.decode(String.self, forKey: .type)
        switch type {
        case "Created":
            self = .created(try container.decode(User.self, forKey: .value))
        case "Updated":
            self = .updated(try container.decode(User.self, forKey: .value))
        case "Deleted":
            self = .deleted
        case "StatusChanged":
            self = .statusChanged(try container.decode(UserStatusChange.self, forKey: .value))
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
        case .deleted:
            try container.encode("Deleted", forKey: .type)
        case .statusChanged(let value):
            try container.encode("StatusChanged", forKey: .type)
            try container.encode(value, forKey: .value)
        }
    }
}