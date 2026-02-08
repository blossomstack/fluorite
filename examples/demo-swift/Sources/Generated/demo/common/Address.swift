
import Foundation
/// Address type used by multiple domains
public struct Address: Codable, Equatable, Sendable {
    /// Street address line 1
    public let street1: String
    /// Street address line 2 (optional)
    public let street2: String?
    /// City
    public let city: String
    /// State or province
    public let state: String
    /// Postal code
    public let postalCode: String
    /// Country code (ISO 3166-1 alpha-2)
    public let country: String
}