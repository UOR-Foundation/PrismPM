// Acceptance infrastructure only. No server is included in browser releases.
import assert from 'node:assert/strict';
import {createServer} from 'node:http';
import {open} from 'node:fs/promises';
import {constants} from 'node:fs';
import {createRequire} from 'node:module';

const require = createRequire('/opt/prismpm/oracles/package.json');
assert.equal(require('playwright/package.json').version, '1.62.1');
const {chromium} = require('playwright');
const executablePath = process.arch === 'x64'
  ? '/ms-playwright/chromium_headless_shell-1234/chrome-headless-shell-linux64/chrome-headless-shell'
  : '/ms-playwright/chromium_headless_shell-1234/chrome-linux/headless_shell';

export async function withBrowser(callback) {
  const server = createServer(async (request, response) => {
    if (request.method !== 'GET') { response.writeHead(405).end(); return; }
    if (request.url === '/') {
      response.writeHead(200, {'content-type': 'text/html', 'cache-control': 'no-store'})
        .end('<!doctype html><title>Browser primitive acceptance</title>');
      return;
    }
    if (!/^\/[a-z][a-z0-9-]*\.mjs$/.test(request.url ?? '')) { response.writeHead(404).end(); return; }
    let handle;
    try {
      handle = await open(new URL(`.${request.url}`, import.meta.url), constants.O_RDONLY | constants.O_NOFOLLOW);
      const stat = await handle.stat();
      if (!stat.isFile() || stat.size > 1_048_576) { response.writeHead(404).end(); return; }
      const source = await handle.readFile();
      response.writeHead(200, {'content-type': 'text/javascript', 'cache-control': 'no-store'}).end(source);
    } catch { response.writeHead(404).end(); }
    finally { await handle?.close(); }
  });
  let browser;
  const persistentContexts = new Set();
  try {
    await new Promise((resolve, reject) => {
      server.once('error', reject);
      server.listen(0, '127.0.0.1', resolve);
    });
    const baseURL = `http://127.0.0.1:${server.address().port}/`;
    browser = await chromium.launch({headless: true, executablePath});
    assert.equal(browser.version(), '151.0.7922.34');
    const launchPersistentContext = async userDataDir => {
      const context = await chromium.launchPersistentContext(userDataDir, {headless: true, executablePath});
      persistentContexts.add(context);
      context.on('close', () => persistentContexts.delete(context));
      assert.equal(context.browser().version(), '151.0.7922.34');
      return context;
    };
    return await callback({browser, baseURL, launchPersistentContext});
  } finally {
    await Promise.all([...persistentContexts].map(context => context.close()));
    await browser?.close();
    server.closeAllConnections();
    await new Promise(resolve => server.close(resolve));
  }
}
