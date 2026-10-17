import {test} from 'node:test';
import assert from 'node:assert/strict';
import vm from 'node:vm';
import {readFileSync, mkdtempSync, mkdirSync, writeFileSync, copyFileSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {createHash} from 'node:crypto';
import {spawnSync} from 'node:child_process';
const source = name => readFileSync(new URL(`../shared/admin/js/${name}.js`, import.meta.url), 'utf8');
function editor(call) {
  const context = vm.createContext({URL, URLSearchParams, history: {replaceState() {}},
    window: {location: {pathname: '/my/content/edit/', href: 'https://site.test/my/content/edit/?doc_type=post'}},
    Alpine: {store: name => name === 'api' ? {call} : {hasRole: () => false}}});
  vm.runInContext(source('content-editor'), context);
  const form = vm.runInContext('documentForm()', context);
  form.doc_type = 'post'; form.doc = {title: 'Draft', body_md: 'Text'};
  form.schema = {fields: [{key:'title'}, {key:'body_md'}]}; form.$el = {reportValidity: () => true};
  return form;
}
test('save stays in editor, submit waits for saved ID, and submitted work becomes read-only', async () => {
  const calls = [];
  const form = editor(async (key, params, body) => { calls.push({key, params, body}); return {id: 42, status:'draft'}; });
  assert.equal(await form.save(), true);
  assert.equal(form.id, 42);
  assert.equal(form.dirty, false);
  assert.equal(form.returnUrl, '/my/content/');
  assert.equal(await form.submit(), true);
  assert.deepEqual(calls.map(c => c.key), ['content_create', 'content_update', 'content_submit']);
  assert.equal(calls[2].params.id, 42);
  assert.equal(form.canEdit, false);
  assert.equal(form.busy, false);
});
test('failed save never submits or discards edits', async () => {
  const calls = [];
  const form = editor(async key => {calls.push(key); throw new Error('Connection failed');});
  form.savedSnapshot = JSON.stringify({...form.doc, title:'Old title'});
  assert.equal(await form.submit(), false);
  assert.deepEqual(calls, ['content_create']);
  assert.equal(form.doc.title, 'Draft');
  assert.equal(form.dirty, true);
  assert.equal(form.busy, false);
  assert.equal(form.error, 'Connection failed');
});
test('failed submission retains saved draft ID for retry without creating duplicates', async () => {
  const form = editor(async key => {if (key === 'content_submit') throw new Error('Required field'); return {id: 42, status:'draft'};});
  assert.equal(await form.submit(), false);
  assert.equal(form.id, 42);
  assert.equal(form.doc.status, 'draft');
  assert.equal(form.canEdit, true);
});
test('list uses own-content query, keeps filters, and paginates consistently', async () => {
  const calls = [], storage = new Map();
  const drafts = Array.from({length: 45}, (_, id) => ({id, title: `Item ${id}`, doc_type:'post', status: id % 2 ? 'submitted' : 'draft'}));
  const context = vm.createContext({URLSearchParams, requireContentLogin: async () => true, sessionStorage: {getItem:k=>storage.get(k),setItem:(k,v)=>storage.set(k,v)},
    Alpine: {store:()=>({call: async (key, params) => {calls.push({key, params}); return key === 'content_schema' ? {types:{post:{label:'Post'}}} : {drafts};}})}});
  vm.runInContext(source('content-management'), context);
  const list = vm.runInContext('contentWorkspace(true)', context); list.$watch = () => {};
  await list.init();
  assert.equal(calls[1].params.mine, true);
  assert.equal(list.pages, 3); assert.equal(list.visible.length, 20);
  list.page = 3; assert.equal(list.visible.length, 5);
  list.filters.status = 'draft'; list.page = 1; list.persist();
  assert.equal(list.filtered.length, 23);
  const reopened = vm.runInContext('contentWorkspace(true)', context); reopened.$watch = () => {};
  await reopened.init(); assert.equal(reopened.filters.status, 'draft');
  assert.equal(reopened.editorUrl('post', 42), '/my/content/edit/?doc_type=post&id=42');
});
test('build integrity check detects an edited shared copy', () => {
  const root = mkdtempSync(join(tmpdir(), 'shared-admin-check-'));
  mkdirSync(join(root, 'utils'));
  copyFileSync(new URL('../shared/admin/check-shared-admin.cjs', import.meta.url), join(root,'utils/check.cjs'));
  writeFileSync(join(root,'screen.txt'), 'original');
  writeFileSync(join(root,'shared-admin-manifest.json'), JSON.stringify({'screen.txt':createHash('sha256').update('original').digest('hex')}));
  assert.equal(spawnSync(process.execPath, [join(root,'utils/check.cjs')]).status, 0);
  writeFileSync(join(root,'screen.txt'), 'drift');
  assert.notEqual(spawnSync(process.execPath, [join(root,'utils/check.cjs')]).status, 0);
});
test('both sites match the canonical source', () => {
  const result = spawnSync(process.execPath, [new URL('../utils/sync-admin.mjs', import.meta.url).pathname, '--check'], {encoding:'utf8'});
  assert.equal(result.status, 0, result.stderr);
});
test('shared API client sends uploads as multipart rather than JSON', async () => {
  class TestFormData {}
  const requests = [];
  const context = vm.createContext({URLSearchParams, FormData:TestFormData, window:{}, console:{log(){}},
    document:{addEventListener(){}},localStorage:{getItem(){return null;}},
    fetch:async (url, options)=>{
      if (!options) return {ok:true,json:async()=>[{key:'upload',url:'/api/upload/',method:'POST',auth:true}]};
      requests.push(options); return {ok:true,headers:{get:()=> 'application/json'},json:async()=>({url:'/uploads/example.png'})};
    }});
  vm.runInContext(readFileSync(new URL('../alpine_custom.js',import.meta.url),'utf8'),context);
  await vm.runInContext("apiCall('upload', {}, new FormData())",context);
  assert.ok(requests[0].body instanceof TestFormData);
  assert.equal(requests[0].headers['Content-Type'],undefined);
  assert.equal(requests[0].credentials,'same-origin');
});
