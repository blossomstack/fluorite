import * as fs from "fs";
import * as path from "path";

// Import generated types
import {
  User,
  Gender,
  TestUnion,
  AnObject,
} from "../generated/demo";

// Type guards for TestUnion
function isPlainString(union: TestUnion): union is { type: "PlainString" } {
  return union.type === "PlainString";
}

function isAnObject(union: TestUnion): union is { type: "AnObject" } & AnObject {
  return union.type === "AnObject";
}

// JSON types (matching serde serialization format - camelCase)
interface UserJson {
  firstName: string;
  lastName: string;
  age: number;
  gender: "Male" | "Female";
  active: boolean;
  info?: unknown;
}

interface AnObjectJson {
  fieldA: string;
}

type TestUnionJson =
  | { type: "PlainString" }
  | ({ type: "AnObject" } & AnObjectJson);

// Sample data creators (produce JSON-compatible objects)
function createSampleUser(withInfo: boolean = true): UserJson {
  return {
    firstName: "John",
    lastName: "Doe",
    age: 30,
    gender: "Male",
    active: true,
    info: withInfo
      ? {
          hobbies: ["reading", "coding"],
          score: 95.5,
        }
      : undefined,
  };
}

function createSampleUserFemale(): UserJson {
  return {
    firstName: "Jane",
    lastName: "Smith",
    age: 25,
    gender: "Female",
    active: false,
    info: undefined,
  };
}

function createSamplePlainString(): TestUnionJson {
  return {
    type: "PlainString",
  };
}

function createSampleAnObject(): TestUnionJson {
  return {
    type: "AnObject",
    fieldA: "Test field value",
  };
}

// Serialization helpers
function serializeToJson(data: unknown): string {
  return JSON.stringify(data, null, 2);
}

function deserializeFromJson<T>(json: string): T {
  return JSON.parse(json) as T;
}

// Write sample data to files
function writeSampleData(outputDir: string): void {
  if (!fs.existsSync(outputDir)) {
    fs.mkdirSync(outputDir, { recursive: true });
  }

  // Write User samples
  const user1 = createSampleUser(true);
  const user2 = createSampleUserFemale();

  fs.writeFileSync(
    path.join(outputDir, "user_male.json"),
    serializeToJson(user1)
  );
  fs.writeFileSync(
    path.join(outputDir, "user_female.json"),
    serializeToJson(user2)
  );

  // Write TestUnion samples
  const plainString = createSamplePlainString();
  const anObject = createSampleAnObject();

  fs.writeFileSync(
    path.join(outputDir, "union_plain_string.json"),
    serializeToJson(plainString)
  );
  fs.writeFileSync(
    path.join(outputDir, "union_an_object.json"),
    serializeToJson(anObject)
  );

  console.log(`Sample data written to ${outputDir}`);
}

// Read and validate JSON files
function readAndValidate(inputDir: string): void {
  const files = [
    "user_male.json",
    "user_female.json",
    "union_plain_string.json",
    "union_an_object.json",
  ];

  for (const file of files) {
    const filePath = path.join(inputDir, file);
    if (!fs.existsSync(filePath)) {
      console.error(`File not found: ${filePath}`);
      continue;
    }

    const content = fs.readFileSync(filePath, "utf-8");

    try {
      if (file.startsWith("user")) {
        const user = deserializeFromJson<UserJson>(content);
        console.log(`Validated User: ${user.firstName} ${user.lastName}`);
        console.log(`  Gender: ${user.gender}, Active: ${user.active}`);
        if (user.info) {
          console.log(`  Info: ${JSON.stringify(user.info)}`);
        }
      } else if (file.startsWith("union")) {
        const union = deserializeFromJson<TestUnionJson>(content);
        console.log(`Validated TestUnion type: ${union.type}`);
        if (isPlainString(union as TestUnion)) {
          console.log(`  PlainString variant (no data)`);
        } else if (isAnObject(union as TestUnion)) {
          console.log(`  Field A: ${(union as { fieldA: string }).fieldA}`);
        }
      }
    } catch (error) {
      console.error(`Failed to validate ${file}:`, error);
      process.exit(1);
    }
  }
}

// Main CLI
function main(): void {
  const args = process.argv.slice(2);

  if (args.length === 0) {
    console.log("Usage: ts-node index.ts [--write <dir> | --read <dir>]");
    console.log("  --write <dir>  Write sample JSON files to directory");
    console.log("  --read <dir>   Read and validate JSON files from directory");
    process.exit(0);
  }

  const command = args[0];
  const dir = args[1] || "./fixtures";

  switch (command) {
    case "--write":
      writeSampleData(dir);
      break;
    case "--read":
      readAndValidate(dir);
      break;
    default:
      console.error(`Unknown command: ${command}`);
      process.exit(1);
  }
}

main();
