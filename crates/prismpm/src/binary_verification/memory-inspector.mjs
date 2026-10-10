// Bounded binary memory/import preflight. Actual WebAssembly compilation follows.
export function inspectBinaryModule(input,maximumPages){
 if(!(input instanceof Uint8Array)||input.length<8||input.length>64*1024*1024||!Number.isInteger(maximumPages)||maximumPages<1||maximumPages>32767)throw new TypeError('invalid-binary-module');
 const bytes=Uint8Array.from(input),header=[0,97,115,109,1,0,0,0];
 if(!header.every((value,index)=>bytes[index]===value))throw new TypeError('invalid-binary-module');
 let at=8,memory,sections=0;
 function unsigned(end){
  let value=0;
  for(let index=0;index<5;index++){
   if(at>=end)throw new TypeError('invalid-binary-module');
   const byte=bytes[at++];
   if(index===4&&(byte&0xf0))throw new TypeError('invalid-binary-module');
   value+=(byte&127)*2**(7*index);
   if(!(byte&128)){if(index>0&&byte===0)throw new TypeError('invalid-binary-module');return value;}
  }
  throw new TypeError('invalid-binary-module');
 }
 while(at<bytes.length){
  if(++sections>64)throw new TypeError('invalid-binary-module');
  const id=bytes[at++],length=unsigned(bytes.length),end=at+length;
  if(end>bytes.length)throw new TypeError('invalid-binary-module');
  if(id===2){if(unsigned(end)!==0||at!==end)throw new TypeError('invalid-binary-module');}
  if(id===5){
   if(memory!==undefined||unsigned(end)!==1||unsigned(end)!==1)throw new TypeError('invalid-binary-module');
   const initialPages=unsigned(end),declaredMaximum=unsigned(end);
   if(initialPages>declaredMaximum||declaredMaximum!==maximumPages||at!==end)throw new TypeError('invalid-binary-module');
   memory={initialPages,maximumPages:declaredMaximum};
  }
  at=end;
 }
 if(memory===undefined)throw new TypeError('invalid-binary-module');
 return memory;
}
