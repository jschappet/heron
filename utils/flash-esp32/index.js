#!/usr/bin/env node
'use strict';
const fs = require('node:fs');
const path = require('node:path');
const os = require('node:os');
const { spawnSync } = require('node:child_process');
const repo = path.resolve(__dirname, '../..');
const help = `Usage: node utils/flash-esp32 --sensor touch|temperature --env DEV|PROD PORT
       node utils/flash-esp32 --sensor touch|temperature --env DEV|PROD --compile-only
       node utils/flash-esp32 --list

Select firmware/config/<sensor>.<env>.json. Flags override SENSOR_TYPE and FLASH_ENV.
ESP32_CONFIG_DIR overrides the config directory. No profile fallback.
Config/env: DISCOVERY_URL, DEVICE_TOKEN, ROOT_CA or ROOT_CA_FILE, REPORT_INTERVAL_SECONDS (default 300).
Tools: ARDUINO_CLI, ARDUINO_SKETCH, ARDUINO_FQBN.
Temperature profiles require SENSOR_MODEL (DHT11/DHT22) and SENSOR_PIN. Close the serial monitor before flashing.`;
function parseArgs(args, env) {
  const options = { sensor: env.SENSOR_TYPE, environment: env.FLASH_ENV, port: null };
  for (let i = 0; i < args.length; i++) {
    const arg = args[i];
    if (arg === '--sensor' || arg === '--env') {
      const value = args[++i];
      if (!value || value.startsWith('-')) throw new Error(`Missing value for ${arg}`);
      options[arg === '--sensor' ? 'sensor' : 'environment'] = value;
    } else if (arg === '--help' || arg === '-h') options.help = true;
    else if (arg === '--list') options.list = true;
    else if (arg === '--compile-only') options.compileOnly = true;
    else if (arg.startsWith('-')) throw new Error(`Unknown option: ${arg}`);
    else if (options.port) throw new Error('Supply only one serial port');
    else options.port = arg;
  }
  if (options.help) return options;
  if (options.list) {
    if (options.port || options.compileOnly) throw new Error('--list cannot be combined with a port or --compile-only');
    return options;
  }
  options.environment = (options.environment || '').toLowerCase();
  if (!['dev', 'prod'].includes(options.environment)) throw new Error('Select --env DEV or PROD (or set FLASH_ENV).');
  if (!['touch', 'temperature'].includes(options.sensor)) throw new Error('Select --sensor touch or temperature.');
  if (options.compileOnly && options.port) throw new Error('Do not supply a port with --compile-only');
  if (!options.compileOnly && !options.port) throw new Error('Supply a serial port or use --compile-only');
  return options;
}
const expand = value => value.startsWith('~/') ? path.join(os.homedir(), value.slice(2)) : value;
function loadConfig(configPath, env, sensor = 'touch') {
  if (!fs.existsSync(configPath)) throw new Error(`Missing selected config: ${configPath}`);
  let config;
  try { config = JSON.parse(fs.readFileSync(configPath, 'utf8')); }
  catch { throw new Error('Unable to read selected config as JSON.'); }
  if (!config || Array.isArray(config) || typeof config !== 'object') throw new Error('Config must be a JSON object');
  const setting = key => {
    const value = env[key] ?? config[key];
    if (typeof value !== 'string') throw new Error(`${key} must be a string`);
    return value;
  };
  const discovery = setting('DISCOVERY_URL');
  let url;
  try { url = new URL(discovery); } catch { throw new Error('DISCOVERY_URL must be a full HTTPS endpoint URL'); }
  if (url.protocol !== 'https:' || !url.hostname || url.username || url.password || url.hash)
    throw new Error('DISCOVERY_URL must be HTTPS without credentials or fragment');
  const schema = sensor === 'temperature' ? 'temperature' : 'touch_count';
  if (config.SENSOR_TYPE !== undefined && config.SENSOR_TYPE !== schema)
    throw new Error('Config SENSOR_TYPE does not match the selected sensor');
  const token = setting('DEVICE_TOKEN');
  if (!token.trim() || /[\x00-\x1f\x7f]/.test(token)) throw new Error('DEVICE_TOKEN must be nonempty without control characters');
  let ca;
  if (env.ROOT_CA !== undefined) ca = env.ROOT_CA;
  else if (env.ROOT_CA_FILE !== undefined) ca = fs.readFileSync(path.resolve(expand(env.ROOT_CA_FILE)), 'utf8');
  else if (config.ROOT_CA) ca = setting('ROOT_CA');
  else ca = fs.readFileSync(path.resolve(path.dirname(configPath), expand(setting('ROOT_CA_FILE'))), 'utf8');
  if (typeof ca !== 'string' || !ca.includes('-----BEGIN CERTIFICATE-----') || !ca.includes('-----END CERTIFICATE-----'))
    throw new Error('ROOT_CA must contain a PEM certificate');
  const values = { DISCOVERY_URL: discovery, SENSOR_TYPE: schema, DEVICE_TOKEN: token, ROOT_CA: ca };
  const rawInterval = env.REPORT_INTERVAL_SECONDS ?? config.REPORT_INTERVAL_SECONDS ?? 300;
  const interval = typeof rawInterval === 'number' ? rawInterval : (/^\d+$/.test(rawInterval || '') ? Number(rawInterval) : NaN);
  if (!Number.isInteger(interval) || interval < 5 || interval > 86400)
    throw new Error('REPORT_INTERVAL_SECONDS must be an integer from 5 to 86400');
  values.REPORT_INTERVAL_SECONDS = interval;
  if (sensor === 'temperature') {
    const model = setting('SENSOR_MODEL');
    const rawPin = env.SENSOR_PIN ?? config.SENSOR_PIN;
    const pin = typeof rawPin === 'number' ? rawPin : (/^\d+$/.test(rawPin || '') ? Number(rawPin) : NaN);
    if (!['DHT11', 'DHT22'].includes(model)) throw new Error('SENSOR_MODEL must be DHT11 or DHT22');
    if (![0, 2, 4, 5, 12, 13, 14, 15, 16, 17, 18, 19, 21, 22, 23, 25, 26, 27, 32, 33].includes(pin))
      throw new Error('SENSOR_PIN must be an output-capable GPIO on the original ESP32');
    values.SENSOR_MODEL = model === 'DHT11' ? 11 : 22;
    values.SENSOR_PIN = pin;
  }
  return values;
}
// Fixed-width octal escapes preserve UTF-8 bytes and cannot consume following digits.
function cppString(value) {
  return '"' + [...Buffer.from(value, 'utf8')].map(byte => {
    if (byte === 34 || byte === 92) return '\\' + String.fromCharCode(byte);
    if (byte >= 32 && byte < 127) return String.fromCharCode(byte);
    return '\\' + byte.toString(8).padStart(3, '0');
  }).join('') + '"';
}
function findCli(env) {
  if (env.ARDUINO_CLI) return env.ARDUINO_CLI;
  for (const directory of (env.PATH || '').split(path.delimiter)) {
    const candidate = path.join(directory, process.platform === 'win32' ? 'arduino-cli.exe' : 'arduino-cli');
    try { fs.accessSync(candidate, fs.constants.X_OK); return candidate; } catch { /* next */ }
  }
  const bundled = '/Applications/Arduino IDE.app/Contents/Resources/app/lib/backend/resources/arduino-cli';
  if (fs.existsSync(bundled)) return bundled;
  throw new Error('Arduino CLI not found. Set ARDUINO_CLI to its executable path.');
}
function run(cli, args, env) {
  const result = spawnSync(cli, args, { env, stdio: 'inherit', shell: false });
  if (result.error) throw new Error('Unable to start Arduino CLI. Check ARDUINO_CLI and permissions.');
  if (result.status !== 0) {
    const error = new Error(`Arduino ${args[0]} failed${result.signal ? ` (${result.signal})` : ` (exit ${result.status})`}`);
    error.exitCode = result.status || 1;
    throw error;
  }
}
function main(args = process.argv.slice(2), env = process.env) {
  const options = parseArgs(args, env);
  if (options.help) { console.log(help); return; }
  if (options.list) { run(findCli(env), ['board', 'list'], env); return; }
  const configPath = path.resolve(expand(env.ESP32_CONFIG_DIR || path.join(repo, 'firmware/config')),
    `${options.sensor}.${options.environment}.json`);
  const values = loadConfig(configPath, env, options.sensor);
  const cli = findCli(env);
  const source = path.resolve(expand(env.ARDUINO_SKETCH || path.join(repo, options.sensor === 'temperature' ? 'firmware/esp32_temperature' : 'firmware/esp32_sensor')));
  if (!fs.statSync(source).isDirectory()) throw new Error('Sketch path must be a directory');
  const fqbn = env.ARDUINO_FQBN || 'esp32:esp32:esp32';
  const buildRoot = path.join(repo, 'target/arduino-touch');
  fs.mkdirSync(buildRoot, { recursive: true });
  const directory = fs.mkdtempSync(path.join(buildRoot, 'flash.'));
  fs.chmodSync(directory, 0o700);
  const staged = path.join(directory, path.basename(source));
  fs.cpSync(source, staged, { recursive: true });
  // Arduino stages the sketch alone, so explicitly include the shared offline implementation.
  for (const file of fs.readdirSync(path.join(repo, 'firmware/shared'))) {
    if (file.endsWith('.h')) fs.copyFileSync(path.join(repo, 'firmware/shared', file), path.join(staged, file));
  }
  const header = path.join(staged, 'device_config.h');
  fs.writeFileSync(header, '// Generated by utils/flash-esp32; do not commit.\n#pragma once\n#include <stdint.h>\n' +
    Object.entries(values).map(([key, value]) => typeof value === 'number' ? `constexpr uint32_t ${key} = ${value};\n` : `const char* ${key} = ${cppString(value)};\n`).join(''), { mode: 0o600 });
  fs.chmodSync(header, 0o600);
  console.log(`Compiling ${options.sensor} (${options.environment}) using ${configPath} for ${fqbn}`);
  const build = path.join(directory, 'build');
  run(cli, ['compile', '--fqbn', fqbn, '--build-path', build, staged], env);
  if (options.compileOnly) { console.log('Compilation complete. No firmware uploaded.'); return; }
  console.log(`Flashing ${options.port}`);
  run(cli, ['upload', '--fqbn', fqbn, '--port', options.port, '--input-dir', build, staged], env);
  console.log(`Flash complete. Firmware reset requested on ${options.port}.`);
}
if (require.main === module) {
  try { main(); } catch (error) {
    console.error(error.message);
    process.exitCode = error.exitCode || 1;
  }
}
module.exports = { parseArgs, loadConfig, cppString, main };
