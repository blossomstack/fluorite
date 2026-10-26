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
assert.deepEqual(d.decodeAnyList([null, 1, 'text']), [null, 1, 'text']);
assert.throws(() => d.decodeAnyList({}), TypeError);
assert.deepEqual(d.decodeAnyMap({one: null}), {one: null});
let count = 10;
function reject(value, path) { assert.throws(() => d.decodeEntry(value), error => error instanceof TypeError && error.message.includes(path)); count++; }
reject(null, 'Entry'); reject([], 'Entry');
for (const key of Object.keys(valid)) { const v = {...valid}; delete v[key]; reject(v, key); }
for (const [key, value] of [['name',1],['count',1.5],['count',2147483648],['large',-1],['large',9007199254740992],['amount',NaN],['amount',Infinity],['enabled',1],['optionalName',null],['state','Other'],['event',{kind:'Other'}],['event',{kind:'Added'}],['nested',[[{count:-1}]]],['byName',{one:{count:'4'}}]]) reject({...valid,[key]:value}, key);
assert.throws(() => d.decodeEntryList({}), TypeError);
assert.throws(() => d.decodeEntryMap([]), TypeError);
console.log(`${count + 2} runtime decoder assertions passed`);

const collisions = require(require('node:path').join(require('node:path').dirname(process.argv[2]), '../collisions/decoders.js'));
const pair = {nested: {count: 3}, flat: {label: 'distinct package'}};
assert.deepEqual(collisions.decodePair(pair), pair);
assert.throws(() => collisions.decodePair({...pair, nested: {count: -1}}), TypeError);
assert.throws(() => collisions.decodePair({...pair, flat: {label: 42}}), TypeError);
assert.deepEqual(collisions.decodeEmpty({}), {});
assert.throws(() => collisions.decodeEmpty(null), TypeError);
assert.throws(() => collisions.decodeEmpty([]), TypeError);
assert.throws(() => collisions.decodeEmpty(1), TypeError);
console.log('7 package collision and empty struct assertions passed');

const strict = process.argv[3] === 'True';
function extra(decoder, value) {
  if (strict) assert.throws(() => decoder(value), TypeError);
  else assert.deepEqual(decoder(value), value);
}
extra(d.decodeEntry, {...valid, surprise: 1});
extra(d.decodeEntry, {...valid, nested: [[{count: 3, surprise: 1}]]});
extra(d.decodeEntry, {...valid, byName: {allowedMapKey: {count: 3, surprise: 1}}});
extra(d.decodeEntry, {...valid, event: {kind: 'Empty', payload: 'not declared'}});
extra(d.decodeEntry, {...valid, event: {kind: 'Added', payload: {count: 1}, surprise: 1}});
extra(collisions.decodeEmpty, {surprise: 1});
extra(d.decodeRenamed, {wireName: 'valid', internalName: 'extra'});
assert.deepEqual(d.decodeRenamed({wireName:'valid'}), {wireName:'valid'});
assert.deepEqual(d.decodeAnyMap({arbitraryKey:{arbitraryValue:1}}), {arbitraryKey:{arbitraryValue:1}});
console.log(`9 strict/default object boundary assertions passed (strict=${strict})`);
