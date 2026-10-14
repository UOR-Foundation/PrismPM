// Real tiny stored ZIP/OCI bytes, not an SDK or authentic provider evidence.
import {gzipSync} from 'node:zlib';
import {fixture,bytes,sha} from './sdk-source-ci-contract-fixture.mjs';
import {storedZip,tarEntry} from './sdk-construction-archive-fixture.mjs';
import {validateSourceCiContract} from './sdk-source-ci-contract.mjs';

export function sourceArchiveFixture({wide=false,entries=rows=>rows,tar=data=>data,zip=()=>{}}={}){
 const f=fixture(),layer=gzipSync(Buffer.alloc(1024)),config=f.files.get('image-config.json');
 const manifest=bytes({schemaVersion:2,mediaType:'application/vnd.oci.image.manifest.v1+json',
  config:{mediaType:'application/vnd.oci.image.config.v1+json',digest:sha(config),size:config.length},
  layers:[{mediaType:'application/vnd.oci.image.layer.v1.tar+gzip',digest:sha(layer),size:layer.length}]});
 const index=bytes({schemaVersion:2,mediaType:'application/vnd.oci.image.index.v1+json',manifests:[{
  mediaType:'application/vnd.oci.image.manifest.v1+json',digest:sha(manifest),size:manifest.length,platform:{os:'linux',architecture:'amd64'}}]});
 const rows=entries([{name:'blobs/',type:'5'},{name:'blobs/sha256/',type:'5'},
  ...[layer,config,manifest].map(data=>({name:'blobs/sha256/'+sha(data).slice(7),data})),
  {name:'index.json',data:index},{name:'oci-layout',data:bytes({imageLayoutVersion:'1.0.0'})}]);
 const archive=tar(Buffer.concat([...rows.map(r=>tarEntry(r.name,r.data,r.type)),Buffer.alloc(1024)]));
 f.files.set('image-manifest.json',manifest);f.envelope.archive={path:'sdk.oci.tar',byte_length:archive.length,digest:sha(archive)};f.seal();
 const members=new Map([['source-ci.json',bytes(f.envelope)],...f.files]);
 const zipped=storedZip(new Map([...members,['sdk.oci.tar',archive]]),wide);zip(zipped);
 f.provider.artifacts[0].size_in_bytes=zipped.bytes.length;f.provider.artifacts[0].digest=sha(zipped.bytes);
 const handle=validateSourceCiContract(bytes(f.provider),bytes(f.envelope),f.files,f.expected,f.now);
 return {...zipped,archive,members,handle,original:f};
}
