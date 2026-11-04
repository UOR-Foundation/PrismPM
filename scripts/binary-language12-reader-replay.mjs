// A conformance bridge over genuine retained language-1.2 compiler output.
import assert from 'node:assert/strict';
import {rejectRuntimeAuditMutations} from './binary-sdk-check.mjs';
const [project,encodedReceipt,archive_sha256,toolchain]=process.argv.slice(2);
assert.equal(process.argv.length,6);
const rejected=rejectRuntimeAuditMutations(project,JSON.parse(encodedReceipt),{archive_sha256,toolchain});
console.log(JSON.stringify({schema:'prismpm/binary-language-1.2-reader-regression/1',real_baseline:'passed',mutations_rejected:rejected,restored_baseline:'passed'}));
