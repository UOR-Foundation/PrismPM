import assert from 'node:assert/strict';
export function verifyRuntime(build,vectors){
 const bytes=build.wasmBytes;
 const module=new WebAssembly.Module(bytes),fresh=()=>new WebAssembly.Instance(module,{});
 assert.deepEqual(WebAssembly.Module.imports(module),[]);
 const growth=fresh(),initial=growth.exports.memory.buffer.byteLength/65536;assert.equal(growth.exports.memory.grow(512-initial),initial);assert.equal(growth.exports.memory.buffer.byteLength,512*65536);assert.throws(()=>growth.exports.memory.grow(1),RangeError,'compiled32MiB maximum');
 const allocation=fresh(),pointer=allocation.exports.holo_alloc(1166279)>>>0;assert.ok(pointer+1166279<=allocation.exports.memory.buffer.byteLength);assert.throws(()=>fresh().exports.holo_alloc(1166280),WebAssembly.RuntimeError);
 const rows=vectors.map(v=>({id:v.id,input:v.request,output:v.response}));assert.equal(rows.length,62);
 assert.equal(Math.max(...rows.filter(v=>v.id!=='OverAllocation').map(v=>v.input.length)),1166279);assert.equal(Math.max(...rows.map(v=>v.output.length)),66803);
 const invoke=input=>{const instance=fresh(),pointer=instance.exports.holo_alloc(input.length);new Uint8Array(instance.exports.memory.buffer,pointer,input.length).set(input);const packed=BigInt.asUintN(64,instance.exports.holo_run(pointer,input.length));return Buffer.from(new Uint8Array(instance.exports.memory.buffer,Number(packed>>32n),Number(packed&0xffffffffn)));};
 const u16=value=>Buffer.from([value>>>8,value&255]);
 for(const[table,total]of[['Members',64],['Messages',256]]){
  const start=rows.find(row=>row.id==='Maximum'+table+'0'),headLength=start.input.readUIntBE(100,3),stateLength=start.input.readUIntBE(103,3),at=106+headLength+stateLength;
  const prefix=start.input.subarray(0,at),intentPrefix=start.input.subarray(at+2,at+35),collected=[];let cursor=Buffer.alloc(0),offset=0;
  do{const intent=Buffer.concat([intentPrefix,u16(cursor.length),cursor]),input=Buffer.concat([prefix,u16(intent.length),intent]);const response=invoke(input);
   assert.equal(response[0],0);assert.equal(response.readUInt16BE(66),total);assert.equal(response.readUInt16BE(68),offset);assert.equal(response[70],16);
   const size=response.readUInt16BE(71),payload=73+size;cursor=response.subarray(73,payload);assert.equal(response.readUIntBE(payload,3),response.length-payload-3);collected.push(response.subarray(payload+3));offset+=16;
   assert.ok(offset<=total);assert.equal(cursor.length,offset===total?0:135);
  }while(cursor.length);
  assert.equal(offset,total);const state=start.input.subarray(106+headLength,106+headLength+stateLength),memberLength=state.readUInt16BE(101),messageLength=state.readUIntBE(103,3);
  const expected=table==='Members'?Buffer.concat([state.subarray(33,65),Buffer.from([0]),state.subarray(108,108+memberLength)]):state.subarray(108+memberLength,108+memberLength+messageLength);
  assert.ok(Buffer.concat(collected).equals(expected),'complete actual-cursor '+table+' traversal without duplication/truncation');
 }
}
