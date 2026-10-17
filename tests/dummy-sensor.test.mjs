import { test } from 'node:test';
import assert from 'node:assert/strict';
import { generateReports, postReports } from '../utils/dummy-sensor.mjs';

const options = { end: '2026-10-15T12:00:00Z' };
test('seven days of valid temperature/humidity reports with repeatable IDs', () => {
  const reports = generateReports(options);
  assert.equal(reports.length, 2017);
  assert.equal(reports.at(-1).reading.observed_at_unix - reports[0].reading.observed_at_unix, 7 * 86400);
  assert.deepEqual(reports, generateReports(options));
  assert.notEqual(reports[0].reading.report_id, generateReports({ ...options, seed: 'another' })[0].reading.report_id);
  assert.equal(new Set(reports.map(r => r.reading.report_id)).size, reports.length);
  for (const { reading: r } of reports) {
    assert.equal(r.schema, 'temperature');
    assert.equal(r.schema_version, 1);
    assert.equal(r.window_end_ms - r.window_start_ms, 300000);
    assert.ok(r.window_end_ms <= 0xffffffff);
    assert.ok(r.data.temperature_c >= -40 && r.data.temperature_c <= 80);
    assert.ok(r.data.humidity_percent >= 0 && r.data.humidity_percent <= 100);
  }
});
test('rejects invalid ranges before sending', () => {
  for (const bad of [{days:-1},{days:Infinity},{interval:0},{interval:1.5},{end:'invalid'},{deviceId:''},{days:30,interval:1}]) {
    assert.throws(() => generateReports({...options,...bad}));
  }
});
test('posts full registration and historical reading using the URL hostname; stops on failure', async () => {
  const reports = generateReports(options).slice(0, 3);
  const calls=[];
  await assert.rejects(postReports({url:'http://workshop.localhost:8582',token:'test-only',reports,
    fetchImpl:async(url,request)=>{calls.push({url,request});return {ok:calls.length===1,status:401};}
  }), /Report 2\/3 failed \(HTTP 401\)/);
  assert.equal(calls.length,2);
  assert.equal(calls[0].request.headers.Authorization,'Bearer test-only');
  assert.equal(calls[0].url.href,'http://workshop.localhost:8582/api/iot');
  assert.equal(calls[0].request.headers.Host,undefined);
  assert.equal(calls[0].request.method,'POST');
  assert.equal(calls[0].request.redirect,'error');
  assert.deepEqual(JSON.parse(calls[0].request.body),reports[0]);
  let completed=0;
  await postReports({url:'http://localhost/api/iot',token:'test-only',reports,fetchImpl:async()=>({ok:true}),progress:n=>{completed=n;}});
  assert.equal(completed,3);
});

test('accepts a trailing slash or an explicit custom endpoint', async () => {
  for (const [url, expected] of [
    ['http://workshop.localhost:8582/', 'http://workshop.localhost:8582/api/iot'],
    ['http://workshop.localhost:8582/custom/iot', 'http://workshop.localhost:8582/custom/iot'],
  ]) {
    await postReports({url, token:'test-only', reports:generateReports(options).slice(0,1),
      fetchImpl:async target=>{assert.equal(target.href,expected);return {ok:true};}});
  }
});
