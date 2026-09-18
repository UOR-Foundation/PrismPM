import assert from 'node:assert/strict';
import {captureLabels,presentation} from '../../sdk/browser/view-dom.mjs';
import {literalLabels} from './label-model.mjs';
export function verifyTextBoundary(){
  const original='\uFEFFplain text',body=Buffer.from(original),frame=Buffer.alloc(84+66+body.length);
  frame.set([0,0x50,0x56,0x4e,1]);frame[7]=1;frame[79]=1;
  frame.writeUIntBE(66+body.length,81,3);frame.fill(1,84,116);frame.fill(2,116,148);
  frame.writeUInt16BE(body.length,148);body.copy(frame,150);
  let bomRejected=false;
  try{captureLabels(new Uint8Array([0xef,0xbb,0xbf,...Buffer.from(literalLabels)]));}
  catch(error){assert.equal(error.code,'invalid-labels');bomRejected=true;}
  assert.deepEqual({bomRejected,message:presentation(frame).rows[0][2]},
    {bomRejected:true,message:original},'BOM is not JSON framing and a leading U+FEFF is real message content');
}
