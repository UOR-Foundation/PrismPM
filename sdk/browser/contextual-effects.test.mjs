import test from 'node:test';
import {verifyContextualEffects} from '../../tests/browser-contextual-effects/checks.mjs';

test('DK-28 exact generated request staging and captured contextual completion',
  {timeout: 3500000}, async t => verifyContextualEffects(t));
