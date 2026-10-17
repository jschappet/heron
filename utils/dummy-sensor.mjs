#!/usr/bin/env node
// Node 20+; no external dependencies. Uses the firmware API, not browser login.
import { parseArgs } from 'node:util';
import { createHash } from 'node:crypto';
import { pathToFileURL } from 'node:url';

export function generateReports({ days = 7, interval = 5, end = new Date().toISOString(), seed = 'workshop-demo', deviceId = 'dummy-temperature' } = {}) {
  if (!Number.isFinite(days) || days <= 0 || days > 30) throw Error('days must be greater than 0 and at most 30');
  if (!Number.isInteger(interval) || interval < 1 || interval > 1440) throw Error('interval must be 1–1440 minutes');
  if (!deviceId.trim() || Buffer.byteLength(deviceId) > 80 || /[\x00-\x1f\x7f]/.test(deviceId)) throw Error('Invalid device ID');
  const endSeconds = Math.floor(Date.parse(end) / 1000);
  const step = interval * 60;
  const count = Math.floor(days * 86400 / step) + 1;
  if (!Number.isFinite(endSeconds) || endSeconds - days * 86400 <= 0) throw Error('end must be a valid date after the historical period');
  if (count > 10000) throw Error('Too many reports; increase interval (maximum 10,000)');
  const digest = createHash('sha256').update(JSON.stringify({ days, interval, endSeconds, seed, deviceId })).digest();
  let randomState = digest.readUInt32LE(0);
  const random = () => { randomState = (Math.imul(1664525, randomState) + 1013904223) >>> 0; return randomState / 4294967296; };
  const bootId = `dummy-${digest.toString('hex').slice(0, 24)}`;
  return Array.from({ length: count }, (_, i) => {
    const observed = endSeconds - (count - 1 - i) * step;
    const daily = Math.sin(observed / 86400 * 2 * Math.PI);
    return {
      device_id: deviceId, name: 'Dummy temperature / humidity (simulated)',
      ip_address: '127.0.0.1', port: 80,
      deployed_code: { name: 'dummy-sensor', version: '1.0.0' },
      capabilities: ['temperature'],
      reading: {
        schema: 'temperature', schema_version: 1,
        report_id: `${bootId}-${i}`, boot_id: bootId,
        observed_at_unix: observed, time_quality: 'estimated',
        window_start_ms: i * step * 1000,
        window_end_ms: (i + 1) * step * 1000,
        window_duration_ms: step * 1000,
        data: { gpio: 4,
          temperature_c: Number((21 + 3 * daily + (random() - .5) * 1.2).toFixed(2)),
          humidity_percent: Number((55 - 10 * daily + (random() - .5) * 6).toFixed(2)) }
      }
    };
  });
}

export async function postReports({ url, token, reports, fetchImpl = fetch, progress = () => {} }) {
  const target = new URL(url);
  if (!['http:', 'https:'].includes(target.protocol) || target.username || target.password) throw Error('Use an HTTP(S) endpoint without embedded credentials');
  if (target.pathname === '/') target.pathname = '/api/iot';
  if (!token?.trim()) throw Error('Set IOT_DEVICE_TOKEN to Heron’s configured device token');
  for (const [index, report] of reports.entries()) {
    const response = await fetchImpl(target, {
      method: 'POST', redirect: 'error', signal: AbortSignal.timeout(15000),
      headers: { 'Content-Type': 'application/json', Authorization: `Bearer ${token}` },
      body: JSON.stringify(report)
    });
    if (!response.ok) throw Error(`Report ${index + 1}/${reports.length} failed (HTTP ${response.status}). Check the endpoint, host, and device token. Earlier reports remain stored; rerun with the same options to resume safely.`);
    progress(index + 1, reports.length);
  }
}

async function main() {
  const { values } = parseArgs({ options: {
    url: { type: 'string' }, days: { type: 'string', default: '7' },
    interval: { type: 'string', default: '5' }, end: { type: 'string' },
    seed: { type: 'string', default: 'workshop-demo' }, 'device-id': { type: 'string', default: 'dummy-temperature' },
    'dry-run': { type: 'boolean' }, help: { type: 'boolean' }
  } });
  if (values.help) {
    console.log(`Usage: IOT_DEVICE_TOKEN=... node utils/dummy-sensor.mjs --url http://workshop.localhost:8582
Options: --days 7 --interval 5 (minutes) --end ISO_DATE --seed STRING --device-id ID
         --dry-run
A site URL posts to /api/iot; an explicit endpoint path is also supported.
Heron selects the site using the URL hostname.
Defaults: 7 days, 5-minute readings, ending now. Dry-run needs no token or URL.
Use the printed --end with the same other options to repeat without duplicate metrics.`);
    return;
  }
  const end = values.end ?? new Date().toISOString();
  const reports = generateReports({ days: Number(values.days), interval: Number(values.interval), end, seed: values.seed, deviceId: values['device-id'] });
  console.log(`${reports.length} simulated reports for ${values['device-id']}. End: ${end}`);
  console.log(`For repeatable retries, use --end ${end} and the same seed/device/interval/days.`);
  if (values['dry-run']) {
    console.log(JSON.stringify({ first: reports[0], last: reports.at(-1) }, null, 2));
    return;
  }
  if (!values.url) throw Error('Supply --url with the target Heron site URL or IoT endpoint, or use --dry-run');
  await postReports({ url: values.url, token: process.env.IOT_DEVICE_TOKEN, reports,
    progress: (done, total) => { if (done % 100 === 0 || done === total) console.log(`Posted ${done}/${total}`); }
  });
  console.log('Complete. Select the dummy device on the metrics page and choose Last 7 days.');
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  main().catch(error => { console.error(error.message); process.exitCode = 1; });
}
