// swift-tools-version: 5.9

// The repo root is a Swift package so apps can depend on the runtime that
// generated Swift code imports (`AnyCodable` for `Any` fields) by URL:
//   .package(url: "https://github.com/blossomstack/fluorite", from: "<version>")
// Its sources live in swift-runtime/, which keeps its own manifest for local use.

import PackageDescription

let package = Package(
    name: "Fluorite",
    platforms: [
        .macOS(.v12),
        .iOS(.v15),
        .tvOS(.v15),
        .watchOS(.v8)
    ],
    products: [
        .library(name: "FluoriteRuntime", targets: ["FluoriteRuntime"]),
    ],
    targets: [
        .target(name: "FluoriteRuntime", path: "swift-runtime/Sources/FluoriteRuntime"),
        .testTarget(
            name: "FluoriteRuntimeTests",
            dependencies: ["FluoriteRuntime"],
            path: "swift-runtime/Tests/FluoriteRuntimeTests"
        ),
    ]
)
