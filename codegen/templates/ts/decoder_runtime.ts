
// Group helpers so schemas without maps/lists also pass noUnusedLocals.
class Wire {
  static expect(valid: boolean, path: string, expected: string): asserts valid {
    if (!valid) throw new TypeError(`${path}: expected ${expected}`);
  }
  static object(value: unknown, path: string): Record<string, unknown> {
    Wire.expect(typeof value === 'object' && value !== null && !Array.isArray(value), path, 'object');
    return value as Record<string, unknown>;
  }
  static keys(value: Record<string, unknown>, allowed: readonly string[], path: string): void {
    for (const key of Object.keys(value)) {
      Wire.expect(allowed.includes(key), `${path}.${key}`, 'declared field');
    }
  }
  static array(value: unknown, path: string): unknown[] {
    Wire.expect(Array.isArray(value), path, 'array');
    return value;
  }
  static visitArray(value: unknown, path: string, check: (item: unknown, path: string) => void): void {
    Wire.array(value, path).forEach((item, index) => check(item, `${path}[${index}]`));
  }
  static visitMap(value: unknown, path: string, check: (key: string, item: unknown, path: string) => void): void {
    Object.entries(Wire.object(value, path)).forEach(([key, item]) => check(key, item, `${path}.${key}`));
  }
}
