#!/usr/bin/env node
import {readFile, writeFile, mkdir, readdir} from 'node:fs/promises';
import {resolve, dirname, join} from 'node:path';
import {fileURLToPath} from 'node:url';
import {createHash} from 'node:crypto';
const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const check = process.argv.includes('--check');
const sites = [
  {root: resolve(root, '../regen-skagit'), pages: 'src/content/admin'},
  {root: resolve(root, '../revillage-society'), pages: 'src/admin-content-7f8a9b2d'},
];
const digest = text => createHash('sha256').update(text).digest('hex');
const common = new Map();
for (const [dir, target] of [['components', 'src/_includes/components'], ['js', 'src/assets/js']]) {
  for (const name of await readdir(join(root, 'shared/admin', dir))) {
    common.set(`${target}/${name}`, await readFile(join(root, 'shared/admin', dir, name), 'utf8'));
  }
}
common.set('utils/check-shared-admin.cjs', await readFile(join(root, 'shared/admin/check-shared-admin.cjs'), 'utf8'));
common.set('src/assets/js/alpine_custom.js', await readFile(join(root, 'alpine_custom.js'), 'utf8'));
const page = (title, url, component, personal = false) => `---\nlayout: layouts/admin.njk\ntitle: "${title}"\npermalink: ${url}\neleventyExcludeFromCollections: true\npersonalContent: ${personal}\n---\n{% include "components/${component}.njk" %}\n`;
let failures = [];
for (const site of sites) {
  const files = new Map(common);
  async function addPages(directory, relative = '') {
    for (const entry of await readdir(directory, {withFileTypes: true})) {
      const name = join(relative, entry.name);
      if (entry.isDirectory()) await addPages(join(directory, entry.name), name);
      else files.set(`${site.pages}/${name}`, await readFile(join(directory, entry.name), 'utf8'));
    }
  }
  await addPages(join(root, 'shared/admin/pages'));
  files.set(`${site.pages}/content.njk`, page('Manage Content', '/admin/content/', 'content-list'));
  files.set(`${site.pages}/edit.njk`, page('Edit Content', '/admin/content/edit/', 'content-editor'));
  files.set(`${site.pages}/roles.njk`, page('Manage Roles', '/admin/roles/', 'roles'));
  for (const [name, title, component] of [['index', 'My Content', 'content-list'], ['edit', 'Edit My Content', 'content-editor'], ['new', 'Create Content', 'content-list']]) {
    files.set(`src/content/my/content/${name}.njk`, page(title, `/my/content/${name === 'index' ? '' : name + '/'}`, component, true));
  }
  // Replaces the obsolete SQL-like user/host role picker with the scoped interface.
  files.set(`${site.pages}/users.njk`, page('Manage Roles', '/admin/users/', 'roles'));
  const manifest = Object.fromEntries([...files].map(([path, text]) => [path, digest(text)]));
  files.set('shared-admin-manifest.json', JSON.stringify(manifest, null, 2) + '\n');
  for (const [relative, content] of files) {
    const path = join(site.root, relative);
    if (check) {
      const actual = await readFile(path, 'utf8').catch(() => null);
      if (actual !== content) failures.push(path);
    } else {
      await mkdir(dirname(path), {recursive: true});
      await writeFile(path, content);
    }
  }
}
if (failures.length) { console.error('Shared admin files differ. Run node utils/sync-admin.mjs from Heron:\n' + failures.join('\n')); process.exitCode = 1; }
else console.log(check ? 'Both sites match the shared admin source.' : 'Shared admin synchronized to both sites.');
