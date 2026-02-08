import XCTest
@testable import FluoriteRuntime

final class AnyCodableTests: XCTestCase {

    // MARK: - Decoding Tests

    func testDecodeNull() throws {
        let json = "null".data(using: .utf8)!
        let decoded = try JSONDecoder().decode(AnyCodable.self, from: json)
        XCTAssertEqual(decoded, .null)
    }

    func testDecodeBool() throws {
        let jsonTrue = "true".data(using: .utf8)!
        let jsonFalse = "false".data(using: .utf8)!

        XCTAssertEqual(try JSONDecoder().decode(AnyCodable.self, from: jsonTrue), .bool(true))
        XCTAssertEqual(try JSONDecoder().decode(AnyCodable.self, from: jsonFalse), .bool(false))
    }

    func testDecodeInt() throws {
        let json = "42".data(using: .utf8)!
        let decoded = try JSONDecoder().decode(AnyCodable.self, from: json)
        XCTAssertEqual(decoded, .int(42))
    }

    func testDecodeDouble() throws {
        let json = "3.14".data(using: .utf8)!
        let decoded = try JSONDecoder().decode(AnyCodable.self, from: json)
        XCTAssertEqual(decoded, .double(3.14))
    }

    func testDecodeString() throws {
        let json = "\"hello\"".data(using: .utf8)!
        let decoded = try JSONDecoder().decode(AnyCodable.self, from: json)
        XCTAssertEqual(decoded, .string("hello"))
    }

    func testDecodeArray() throws {
        let json = "[1, \"two\", true]".data(using: .utf8)!
        let decoded = try JSONDecoder().decode(AnyCodable.self, from: json)
        XCTAssertEqual(decoded, .array([.int(1), .string("two"), .bool(true)]))
    }

    func testDecodeObject() throws {
        let json = "{\"name\": \"test\", \"count\": 5}".data(using: .utf8)!
        let decoded = try JSONDecoder().decode(AnyCodable.self, from: json)
        XCTAssertEqual(decoded, .object(["name": .string("test"), "count": .int(5)]))
    }

    func testDecodeNestedStructure() throws {
        let json = """
        {
            "users": [
                {"name": "Alice", "age": 30},
                {"name": "Bob", "age": 25}
            ],
            "metadata": null
        }
        """.data(using: .utf8)!

        let decoded = try JSONDecoder().decode(AnyCodable.self, from: json)
        let expected: AnyCodable = .object([
            "users": .array([
                .object(["name": .string("Alice"), "age": .int(30)]),
                .object(["name": .string("Bob"), "age": .int(25)])
            ]),
            "metadata": .null
        ])
        XCTAssertEqual(decoded, expected)
    }

    // MARK: - Encoding Tests

    func testEncodeNull() throws {
        let value: AnyCodable = .null
        let data = try JSONEncoder().encode(value)
        let json = String(data: data, encoding: .utf8)!
        XCTAssertEqual(json, "null")
    }

    func testEncodeBool() throws {
        let valueTrue: AnyCodable = .bool(true)
        let dataTrue = try JSONEncoder().encode(valueTrue)
        XCTAssertEqual(String(data: dataTrue, encoding: .utf8), "true")

        let valueFalse: AnyCodable = .bool(false)
        let dataFalse = try JSONEncoder().encode(valueFalse)
        XCTAssertEqual(String(data: dataFalse, encoding: .utf8), "false")
    }

    func testEncodeInt() throws {
        let value: AnyCodable = .int(42)
        let data = try JSONEncoder().encode(value)
        XCTAssertEqual(String(data: data, encoding: .utf8), "42")
    }

    func testEncodeDouble() throws {
        let value: AnyCodable = .double(3.14)
        let data = try JSONEncoder().encode(value)
        let json = String(data: data, encoding: .utf8)!
        XCTAssertTrue(json.hasPrefix("3.14"))
    }

    func testEncodeString() throws {
        let value: AnyCodable = .string("hello")
        let data = try JSONEncoder().encode(value)
        XCTAssertEqual(String(data: data, encoding: .utf8), "\"hello\"")
    }

    func testEncodeArray() throws {
        let value: AnyCodable = .array([.int(1), .string("two")])
        let data = try JSONEncoder().encode(value)
        let json = String(data: data, encoding: .utf8)!
        XCTAssertTrue(json.contains("1"))
        XCTAssertTrue(json.contains("\"two\""))
    }

    func testEncodeObject() throws {
        let value: AnyCodable = .object(["key": .string("value")])
        let data = try JSONEncoder().encode(value)
        let json = String(data: data, encoding: .utf8)!
        XCTAssertTrue(json.contains("\"key\""))
        XCTAssertTrue(json.contains("\"value\""))
    }

    // MARK: - Round-trip Tests

    func testRoundTrip() throws {
        let original: AnyCodable = .object([
            "string": .string("hello"),
            "number": .int(42),
            "float": .double(3.14),
            "bool": .bool(true),
            "null": .null,
            "array": .array([.int(1), .int(2), .int(3)]),
            "nested": .object(["inner": .string("value")])
        ])

        let data = try JSONEncoder().encode(original)
        let decoded = try JSONDecoder().decode(AnyCodable.self, from: data)
        XCTAssertEqual(original, decoded)
    }

    // MARK: - Literal Conformance Tests

    func testNilLiteral() {
        let value: AnyCodable = nil
        XCTAssertEqual(value, .null)
    }

    func testBoolLiteral() {
        let value: AnyCodable = true
        XCTAssertEqual(value, .bool(true))
    }

    func testIntLiteral() {
        let value: AnyCodable = 42
        XCTAssertEqual(value, .int(42))
    }

    func testDoubleLiteral() {
        let value: AnyCodable = 3.14
        XCTAssertEqual(value, .double(3.14))
    }

    func testStringLiteral() {
        let value: AnyCodable = "hello"
        XCTAssertEqual(value, .string("hello"))
    }

    func testArrayLiteral() {
        let value: AnyCodable = [1, "two", true]
        XCTAssertEqual(value, .array([.int(1), .string("two"), .bool(true)]))
    }

    func testDictionaryLiteral() {
        let value: AnyCodable = ["key": "value", "count": 5]
        XCTAssertEqual(value, .object(["key": .string("value"), "count": .int(5)]))
    }
}
