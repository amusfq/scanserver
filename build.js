const fs = require('node:fs');

fs.rmSync('dist', { recursive: true, force: true });
fs.mkdirSync('dist');
fs.cpSync('app-ui/dist', 'dist/client', { recursive: true });
fs.copyFileSync('rust-server/target/release/scanserver', 'dist/scanserver');
fs.chmodSync('dist/scanserver', 0o755);
