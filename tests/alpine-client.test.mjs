import { test } from 'node:test';
import assert from 'node:assert/strict';
import vm from 'node:vm';
import { readFileSync } from 'node:fs';
const client = readFileSync(new URL('../alpine_custom.js', import.meta.url), 'utf8');
test('shared client encodes login, preserves cookies, and builds pagination', async () => {
  const requests = [];
  const context = vm.createContext({ URLSearchParams, console: { log() {} }, window: {},
    document: { addEventListener() {} }, localStorage: { getItem() { return null; } },
    fetch: async (url, options) => {
      requests.push({url, options});
      if (!options) return { ok: true, json: async () => [
        {key:'login',url:'/api/auth/login',method:'POST'},
        {key:'iot_device_metrics',url:'/api/iot/devices/{device_id}/metrics',method:'GET'}] };
      return {ok:true, headers: {get:()=> 'application/json'}, json:async()=>({})};
    }
  });
  vm.runInContext(client, context);
  await vm.runInContext("apiCall('login', {}, new URLSearchParams({username:'a+b',password:'p&='}))", context);
  assert.equal(requests[1].options.body, 'username=a%2Bb&password=p%26%3D');
  assert.equal(requests[1].options.credentials, 'same-origin');
  assert.equal(requests[1].options.headers['Content-Type'], 'application/x-www-form-urlencoded');
  await vm.runInContext("apiCall('iot_device_metrics', {device_id:'a/b',limit:500,before_id:7})", context);
  assert.equal(requests[2].url, '/api/iot/devices/a%2Fb/metrics?limit=500&before_id=7');
});
