import {test} from 'node:test';
import assert from 'node:assert/strict';
import vm from 'node:vm';
import {readFileSync} from 'node:fs';
const source = name => readFileSync(new URL(`../shared/admin/js/${name}.js`, import.meta.url), 'utf8');

for (const [script, component] of [['content-management', 'contentWorkspace(true)'], ['content-editor', 'documentForm()']]) {
  test(`${script}: signed-out visitors redirect before requesting protected content`, async () => {
    const calls = [], redirects = [];
    const context = vm.createContext({
      window:{location:{pathname:'/my/content/edit/',replace:url=>redirects.push(url)}},
      Alpine:{store:()=>({call:async key=>{
        calls.push(key);
        assert.equal(key,'profile_json');
        return {authenticated:false};
      }})}
    });
    vm.runInContext(source('content-access') + source(script), context);
    const page = vm.runInContext(component, context);
    await page.init();
    assert.deepEqual(calls,['profile_json']);
    assert.deepEqual(redirects,['/login/']);
    assert.equal(page.signedIn,false);
    assert.ok(!page.error);
  });
}

test('session check waits for the profile response instead of treating a loading session as signed out', async () => {
  const redirects = [];
  let resolveProfile;
  const profile = new Promise(resolve => {resolveProfile = resolve;});
  const context = vm.createContext({window:{location:{replace:url=>redirects.push(url)}},
    Alpine:{store:()=>({call:()=>profile,loading:true,authenticated:false})}});
  vm.runInContext(source('content-access'),context);
  const result = vm.runInContext('requireContentLogin()',context);
  assert.deepEqual(redirects,[]);
  resolveProfile({authenticated:true});
  assert.equal(await result,true);
  assert.deepEqual(redirects,[]);
});

for (const site of ['regen-skagit','revillage-society']) {
  for (const ok of [true,false]) {
    test(`${site}: ${ok ? 'successful login opens My Content' : 'failed login stays on Login'}`, async () => {
      const page = readFileSync(new URL(`../../${site}/src/login/index.njk`,import.meta.url),'utf8');
      const script = page.slice(page.lastIndexOf('<script>')+8,page.lastIndexOf('</script>'));
      const location = {href:'/login/'};
      const context = vm.createContext({URLSearchParams,window:{location},fetch:async()=>({ok,json:async()=>({id:7})})});
      vm.runInContext(script,context);
      const form = vm.runInContext('authApp()',context);
      form.$store = {auth:{}};
      await form.login();
      assert.equal(location.href,ok ? '/my/content/' : '/login/');
      if (!ok) assert.equal(form.message,'Login failed!');
    });
  }
}
for (const failure of ['missing capability','expired session','network failure']) {
  test(`session guard handles ${failure}`, async () => {
    const redirects = [];
    const error = new Error(failure === 'missing capability' ? 'Unknown API function: profile_json' : failure);
    if (failure === 'expired session') error.status = 401;
    const context = vm.createContext({window:{location:{replace:url=>redirects.push(url)}},
      Alpine:{store:()=>({call:async()=>{throw error;}})}});
    vm.runInContext(source('content-access'),context);
    const result = vm.runInContext('requireContentLogin()',context);
    if (failure === 'network failure') {
      await assert.rejects(result,/network failure/);
      assert.deepEqual(redirects,[]);
    } else {
      assert.equal(await result,false);
      assert.deepEqual(redirects,['/login/']);
    }
  });
}
