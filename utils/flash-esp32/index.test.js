'use strict';
const { test } = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const { spawnSync } = require('node:child_process');
const { parseArgs, loadConfig, cppString } = require('./index');

test('profile flags override environment and invalid selections fail', () => {
  const options = parseArgs(['--sensor', 'touch', '--env', 'DEV', '--compile-only'], { SENSOR_TYPE: 'temperature', FLASH_ENV: 'PROD' });
  assert.equal(options.environment, 'dev');
  assert.equal(options.sensor, 'touch');
  assert.equal(parseArgs(['--compile-only'], { SENSOR_TYPE: 'touch', FLASH_ENV: 'PROD' }).environment, 'prod');
  for (const args of [[], ['--sensor'], ['--sensor', 'unknown', '--env', 'dev', '--compile-only'], ['--env', 'staging']])
    assert.throws(() => parseArgs(args, {}));
});

test('configuration precedence, relative CA paths, escaping and missing profiles', t => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'esp32-config-'));
  t.after(() => fs.rmSync(dir, { recursive: true, force: true }));
  const ca = '-----BEGIN CERTIFICATE-----\nexample\n-----END CERTIFICATE-----';
  const config = path.join(dir, 'touch.dev.json');
  fs.writeFileSync(path.join(dir, 'ca.pem'), ca);
  fs.writeFileSync(config, JSON.stringify({ DISCOVERY_URL: 'https://example.com/iot', DEVICE_TOKEN: 'test"token', ROOT_CA_FILE: 'ca.pem' }));
  assert.equal(loadConfig(config, {}).ROOT_CA, ca);
  assert.equal(loadConfig(config, {}).REPORT_INTERVAL_SECONDS, 300);
  assert.equal(loadConfig(config, { REPORT_INTERVAL_SECONDS: '60' }).REPORT_INTERVAL_SECONDS, 60);
  for (const value of ['0', '-1', '1.5', 'no', '86401'])
    assert.throws(() => loadConfig(config, { REPORT_INTERVAL_SECONDS: value }), /REPORT_INTERVAL_SECONDS/);
  const values = loadConfig(config, { DISCOVERY_URL: 'https://other.example/iot', ROOT_CA: ca + '\n' });
  assert.equal(values.DISCOVERY_URL, 'https://other.example/iot');
  assert.equal(values.ROOT_CA, ca + '\n');
  assert.equal(cppString('a"\\\n'), '"a\\"\\\\\\012"');
  assert.throws(() => loadConfig(path.join(dir, 'touch.prod.json'), {}), /Missing selected config/);
  assert.throws(() => loadConfig(config, { DISCOVERY_URL: 'http://example.com' }), /HTTPS/);
});

test('CLI compiles before upload, compile-only never uploads, failed compilation stops', t => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'esp32-cli-'));
  t.after(() => fs.rmSync(dir, { recursive: true, force: true }));
  const cli = path.join(dir, 'mock-cli');
  fs.writeFileSync(cli, `#!${process.execPath}\nrequire('node:fs').appendFileSync(process.env.CLI_LOG, JSON.stringify(process.argv.slice(2))+'\\n');\nif (process.env.FAIL_COMPILE && process.argv[2] === 'compile') process.exit(7);\n`, { mode: 0o700 });
  fs.writeFileSync(path.join(dir, 'touch.dev.json'), JSON.stringify({ DISCOVERY_URL: 'https://example.com/iot', DEVICE_TOKEN: 'test', ROOT_CA: '-----BEGIN CERTIFICATE-----\ntest\n-----END CERTIFICATE-----' }));
  const env = { ...process.env, ARDUINO_CLI: cli, ESP32_CONFIG_DIR: dir, CLI_LOG: path.join(dir, 'calls') };
  for (const key of ['ROOT_CA', 'ROOT_CA_FILE', 'DEVICE_TOKEN', 'DISCOVERY_URL', 'SENSOR_TYPE', 'FLASH_ENV', 'ARDUINO_SKETCH']) delete env[key];
  const run = (args, extra = {}) => spawnSync(process.execPath, [__dirname, '--sensor', 'touch', '--env', 'DEV', ...args], { env: { ...env, ...extra }, encoding: 'utf8' });
  assert.equal(run(['--compile-only']).status, 0);
  assert.equal(run(['/dev/mock-port']).status, 0);
  assert.equal(run(['/dev/mock-port'], { FAIL_COMPILE: '1' }).status, 7);
  const calls = fs.readFileSync(env.CLI_LOG, 'utf8').trim().split('\n').map(JSON.parse);
  assert.deepEqual(calls.map(args => args[0]), ['compile', 'compile', 'upload', 'compile']);
  assert.equal(calls[2][calls[2].indexOf('--port') + 1], '/dev/mock-port');
  assert.equal(calls[1].at(-1), calls[2].at(-1));
});


test('temperature config requires model and GPIO and selects its schema', t => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'esp32-temp-'));
  t.after(() => fs.rmSync(dir, { recursive: true, force: true }));
  const file = path.join(dir, 'temperature.dev.json');
  fs.writeFileSync(file, JSON.stringify({ DISCOVERY_URL: 'https://example.com/iot', DEVICE_TOKEN: 'test', ROOT_CA: '-----BEGIN CERTIFICATE-----\nexample\n-----END CERTIFICATE-----', SENSOR_MODEL: 'DHT11', SENSOR_PIN: 4 }));
  const values = loadConfig(file, {}, 'temperature');
  assert.equal(values.SENSOR_TYPE, 'temperature');
  assert.equal(values.SENSOR_MODEL, 11);
  assert.equal(values.SENSOR_PIN, 4);
  assert.throws(() => loadConfig(file, { SENSOR_PIN: '34' }, 'temperature'), /GPIO/);
  assert.throws(() => loadConfig(file, { SENSOR_MODEL: 'other' }, 'temperature'), /SENSOR_MODEL/);
});
