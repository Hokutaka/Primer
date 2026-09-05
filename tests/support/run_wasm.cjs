// テスト専用ホストです。文字列はメモリを受け取らず、出力バイトだけを収集します。
const fs = require('node:fs');
const output = [];
const writeText = value => output.push(Buffer.from(`${value}\n`, 'utf8'));
const imports = { primer: {
  write_byte(value) {
    if (!Number.isInteger(value) || value < 0 || value > 255) throw new Error('invalid output byte');
    output.push(Buffer.from([value]));
  },
  print_bool: value => writeText(value ? 'true' : 'false'),
  print_i64: value => writeText(value.toString()),
  // この比較fixtureの浮動小数点値は正確に表せる1.5だけです。
  print_f32: value => { if (value !== 1.5) throw new Error('unsupported test float'); writeText(value); },
  print_f64: value => { if (value !== 1.5) throw new Error('unsupported test float'); writeText(value); },
} };
WebAssembly.instantiate(fs.readFileSync(process.argv[2]), imports).then(({ instance }) => {
  if (Object.keys(instance.exports).join(',') !== 'main') throw new Error('unexpected public export');
  instance.exports.main();
  process.stdout.write(Buffer.concat(output));
}).catch(error => { console.error(error.message); process.exitCode = 1; });
