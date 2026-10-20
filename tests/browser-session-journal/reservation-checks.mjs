import assert from 'node:assert/strict';
import {mkdtempSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';

export function createReservationEvidenceDirectory() {
  // Retained evidence belongs to the private suite, outside both the read-only
  // installed source and the build directory retired by the successful owner.
  return mkdtempSync(join(tmpdir(), 'prismpm-reservation-verification-'));
}

export function verifyReservationNativeInventory(output, rows) {
  assert.equal(output, rows.map(row => 'PASS ' + row.id + '\n').join('')
    + 'PASS ' + rows.length + ' journal reservation vectors twice\n',
  'complete exact reservation native inventory');
}
