import test from 'node:test';
import {verifyOperationJournal} from '../../tests/browser-operation-journal/checks.mjs';

test('DK-24 generated durable operation journal and authenticated terminal-receipt recovery',
  {timeout: 3500000}, async t => verifyOperationJournal(t));
