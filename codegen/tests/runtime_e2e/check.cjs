const assert = require('node:assert/strict');
const d = require(process.argv[2]);
const valid = { name: '你好', count: -1, large: 42, amount: 1.5, enabled: true,
  nested: [[{count: 3}]], byName: {one: {count: 4}}, state: 'Ready', event: {kind: 'Added', payload: {count: 1}} };
assert.deepEqual(d.decodeEntry(valid), valid);
assert.deepEqual(d.decodeEntry({...valid, optionalName: 'optional', event: {kind: 'Empty'}}).event, {kind:'Empty'});
assert.deepEqual(d.decodeEntryList([valid]), [valid]);
assert.deepEqual(d.decodeEntryMap({one: valid}), {one: valid});
assert.deepEqual(d.decodeNumericMap({'12': 'twelve'}), {'12':'twelve'});
assert.throws(() => d.decodeNumericMap({'-1':'bad'}), TypeError);
assert.throws(() => d.decodeNumericMap({'bad':'bad'}), TypeError);
let count = 7;
function reject(value, path) { assert.throws(() => d.decodeEntry(value), error => error instanceof TypeError && error.message.includes(path)); count++; }
reject(null, 'Entry'); reject([], 'Entry');
for (const key of Object.keys(valid)) { const v = {...valid}; delete v[key]; reject(v, key); }
for (const [key, value] of [['name',1],['count',1.5],['count',2147483648],['large',-1],['large',9007199254740992],['amount',NaN],['amount',Infinity],['enabled',1],['optionalName',null],['state','Other'],['event',{kind:'Other'}],['event',{kind:'Added'}],['nested',[[{count:-1}]]],['byName',{one:{count:'4'}}]]) reject({...valid,[key]:value}, key);
assert.throws(() => d.decodeEntryList({}), TypeError);
assert.throws(() => d.decodeEntryMap([]), TypeError);
console.log(`${count + 2} runtime decoder assertions passed`);
