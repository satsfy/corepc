cd /home/renato/Desktop/bitcoin/corepc/spec_compare/specs && node - <<'NODE'
const fs = require('fs');
const Ajv = require('ajv');
const addFormats = require('ajv-formats');
const openrpcSchema = require('@open-rpc/meta-schema').openrpcDocument;
const jsonSchemaTools = require('@json-schema-tools/meta-schema').default;

const ajv = new Ajv({
  allErrors: true,
  strict: false,
  validateFormats: true,
  allowUnionTypes: true,
});
addFormats(ajv);

// Register referenced schemas under both forms used by upstream schemas
ajv.addSchema(jsonSchemaTools, 'https://meta.json-schema.tools/');
ajv.addSchema(jsonSchemaTools, 'https://meta.json-schema.tools');
ajv.addSchema(openrpcSchema, 'https://meta.open-rpc.org/');

const validate = ajv.compile(openrpcSchema);
const doc = JSON.parse(fs.readFileSync('v30_2_0_openrpc.json', 'utf8'));
const ok = validate(doc);
if (ok) {
  console.log('VALID: v30_2_0_openrpc.json');
  process.exit(0);
}
console.error('INVALID: v30_2_0_openrpc.json');
for (const err of validate.errors || []) {
  console.error(` - ${err.instancePath || '/'} ${err.message}`);
}
process.exit(1);
NODE
