// Acquire original current-run provider metadata, never infer acceptance from it.
import assert from 'node:assert/strict';
import {execFileSync} from 'node:child_process';
import {writeFileSync} from 'node:fs';

const [output, run] = process.argv.slice(2);
assert.equal(process.argv.length, 4);
assert.equal(process.env.GITHUB_REPOSITORY, 'UOR-Foundation/PrismPM');
assert.match(run, /^[1-9][0-9]{0,19}$/);
assert.equal(run, process.env.GITHUB_RUN_ID);
const acquire = (resource, field) => {
  const bytes = execFileSync('gh', ['api', '--paginate', '--slurp',
    `/repos/UOR-Foundation/PrismPM/actions/runs/${run}/${resource}`],
  {encoding: 'utf8', timeout: 120000, maxBuffer: 16 * 1024 ** 2});
  const pages = JSON.parse(bytes);
  assert(Array.isArray(pages) && pages.length > 0 && pages.length <= 100);
  const rows = pages.flatMap(page => {assert(Array.isArray(page[field])); return page[field];});
  assert(rows.length <= 10000);
  return rows;
};
const metadata = {artifacts: acquire('artifacts?per_page=100', 'artifacts'),
  jobs: acquire('jobs?filter=all&per_page=100', 'jobs')};
writeFileSync(output, JSON.stringify(metadata) + '\n', {flag: 'wx'});
