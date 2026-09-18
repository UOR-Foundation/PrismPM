// Independent fixture bytes: reuse the complete literal Query maximum, never model execution.
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {createHash} from 'node:crypto';
import {corpus} from './query-corpus.mjs';
export function interoperability(){
  const data=JSON.parse(readFileSync(new URL('./query-fixtures.json',import.meta.url)));
  assert.equal(data.schema,'prismpm/workspace-view-query-fixtures/1');assert.equal(data.fixtures.length,4);
  const request=Buffer.from(corpus().find(row=>row.id==='MaximumMembers0').request);
  const headLength=request.readUIntBE(100,3),stateLength=request.readUIntBE(103,3),stateStart=106+headLength;
  const membersStart=stateStart+108;assert.equal(request.readUInt16BE(stateStart+101),63*33);
  const members=Array.from({length:63},(_,i)=>Buffer.from(request.subarray(membersStart+33*i,membersStart+33*(i+1)))).reverse();
  Buffer.concat(members).copy(request,membersStart);
  const prefix=request.subarray(0,106+headLength+stateLength),workspace=request.subarray(stateStart+1,stateStart+33),out=[];
  const u16=value=>{const bytes=Buffer.alloc(2);bytes.writeUInt16BE(value);return bytes;};let cursor=Buffer.alloc(0);
  for(const [index,row]of data.fixtures.entries()){
    assert.equal(row.id,'ActualQueryGrantOrder'+index*16);assert.equal(row.requested_cursor_hex,cursor.toString('hex'));
    const intent=Buffer.concat([Buffer.from([0]),workspace,u16(cursor.length),cursor]),input=Buffer.concat([prefix,u16(intent.length),intent]);
    assert.equal(createHash('sha256').update(input).digest('hex'),row.request_sha256,'every reconstructed original request is byte-exact');
    const response=Buffer.from(row.response_hex,'hex');assert.equal(response[0],0);assert.equal(response[1],0);assert.deepEqual(response.subarray(2,34),workspace);
    assert.equal(response.readUInt16BE(66),64);assert.equal(response.readUInt16BE(68),index*16);assert.equal(response[70],16);
    const length=response.readUInt16BE(71);cursor=response.subarray(73,73+length);
    out.push({id:row.id,request:input,response});
  }
  assert.equal(cursor.length,0);return out;
}
