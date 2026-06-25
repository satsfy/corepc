// test-labeler.js
const filePathsToLabels = {
  "bitcoind/": "C-bitcoind",
  "bitreq/": "C-bitreq",
  "client/": "C-client",
  "electrsd/": "C-electrsd",
  "integration_test/": "C-integration-test",
  "jsonrpc/": "C-jsonrpc",
  "types/": "C-types",
  "verify/": "C-verify",
};

function simulate({ changedFiles, currentLabels }) {
  const labelsToAdd = new Set();
  changedFiles.forEach((f) => {
    for (const [path, label] of Object.entries(filePathsToLabels)) {
      if (f.startsWith(path)) {
        labelsToAdd.add(label);
        break;
      }
    }
  });

  const allKnownLabels = new Set(Object.values(filePathsToLabels));
  const toRemove = currentLabels.filter((l) => allKnownLabels.has(l) && !labelsToAdd.has(l));

  return { add: [...labelsToAdd], remove: toRemove };
}

// Case: PR touches only client/, had C-types applied before
console.log(
  simulate({
    changedFiles: ["client/src/lib.rs"],
    currentLabels: ["C-client", "C-types"],
  }),
);
// Expected: add: ['C-client'], remove: ['C-types']

// Case: unmanaged label (e.g. 'help wanted') must not be removed
console.log(
  simulate({
    changedFiles: ["bitcoind/src/lib.rs"],
    currentLabels: ["C-bitcoind", "help wanted"],
  }),
);
// Expected: add: ['C-bitcoind'], remove: []
