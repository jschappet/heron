// Runs without a neighboring Heron checkout, including isolated hosting builds.
const {readFileSync} = require('node:fs');
const {createHash} = require('node:crypto');
const {resolve} = require('node:path');
const root = resolve(__dirname, '..');
const manifest = JSON.parse(readFileSync(resolve(root, 'shared-admin-manifest.json'), 'utf8'));
const mismatches = Object.entries(manifest).filter(([path, expected]) => {
  try { return createHash('sha256').update(readFileSync(resolve(root, path))).digest('hex') !== expected; }
  catch { return true; }
});
if (mismatches.length) throw new Error('Shared admin drift. Update the canonical source in Heron and run utils/sync-admin.mjs:\n' + mismatches.map(([p]) => p).join('\n'));
console.log('Shared admin integrity verified.');
