// swift-tools-version: 5.9

import PackageDescription

let package = Package(
    name: "FluoriteDemo",
    platforms: [
        .macOS(.v12)
    ],
    dependencies: [
        .package(path: "../../swift-runtime"),
    ],
    targets: [
        .executableTarget(
            name: "Demo",
            dependencies: [
                .product(name: "FluoriteRuntime", package: "swift-runtime"),
            ],
            path: "Sources",
            sources: ["Demo", "Generated"]
        ),
    ]
)
