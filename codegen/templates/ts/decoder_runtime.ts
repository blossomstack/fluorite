
function expect(valid: boolean, path: string, expected: string): asserts valid {
  if (!valid) throw new TypeError(`${path}: expected ${expected}`);
}
function object(value: unknown, path: string): Record<string, unknown> {
  expect(typeof value === 'object' && value !== null && !Array.isArray(value), path, 'object');
  return value as Record<string, unknown>;
}
function array(value: unknown, path: string): unknown[] {
  expect(Array.isArray(value), path, 'array');
  return value;
}

function visitArray(value: unknown, path: string, check: (item: unknown, path: string) => void): void {
  array(value, path).forEach((item, index) => check(item, `${path}[${index}]`));
}
function visitMap(value: unknown, path: string, check: (key: string, item: unknown, path: string) => void): void {
  Object.entries(object(value, path)).forEach(([key, item]) => check(key, item, `${path}.${key}`));
}
