import fs from 'fs';
import https from 'https';
import FormData from 'form-data';

const JWT = process.env.PINATA_JWT;
if (!JWT) {
  console.error('❌ Missing PINATA_JWT environment variable.');
  console.error('   Set it: $env:PINATA_JWT = "your-jwt-here"');
  process.exit(1);
}

const ARCHIVE = 'Genesis-Protocol-v1.0.0-Experimental-Engine.tar.gz';
const EXPECTED_SHA256 = 'd242705e9ba41a842fd80efc5b8aadd9d25bc997d9bd5d8f33a5f96aae8945b8';

if (!fs.existsSync(ARCHIVE)) {
  console.error(`❌ Archive not found: ${ARCHIVE}`);
  process.exit(1);
}

console.log(`📦 Uploading: ${ARCHIVE}`);
console.log(`🔒 Expected SHA-256: ${EXPECTED_SHA256}`);
console.log('');

const formData = new FormData();
formData.append('file', fs.createReadStream(ARCHIVE));

const metadata = JSON.stringify({
  name: 'Genesis-Protocol-v1.0.0-Experimental-Engine',
  keyvalues: {
    version: 'v1.0.0',
    commit: '1955dfa900296065308be5dcd232c580e9e8ef9a',
    sha256: EXPECTED_SHA256,
    author: 'Kevan Burns',
    orcid: '0009-0008-8425-939X',
    doi: '10.5281/zenodo.18646886',
    frozen: '2026-02-22',
    purpose: 'IP priority evidence — frozen experimental engine snapshot'
  }
});
formData.append('pinataMetadata', metadata);

const options = {
  method: 'POST',
  host: 'api.pinata.cloud',
  path: '/pinning/pinFileToIPFS',
  headers: {
    'Authorization': `Bearer ${JWT}`,
    ...formData.getHeaders()
  }
};

const req = https.request(options, (res) => {
  let data = '';
  res.on('data', chunk => data += chunk);
  res.on('end', () => {
    if (res.statusCode === 200) {
      const result = JSON.parse(data);
      console.log('✅ UPLOAD SUCCESS');
      console.log(`   CID:  ${result.IpfsHash}`);
      console.log(`   Size: ${result.PinSize} bytes`);
      console.log(`   URL:  https://gateway.pinata.cloud/ipfs/${result.IpfsHash}`);
      console.log('');
      console.log('📋 Record this CID in IP_RECORD.md');
      console.log(`   ipfs://${result.IpfsHash}`);
    } else {
      console.error(`❌ Upload failed — HTTP ${res.statusCode}`);
      console.error(data);
    }
  });
});

req.on('error', err => {
  console.error('❌ Network error:', err.message);
});

formData.pipe(req);
