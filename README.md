A template of writing RISC-V DOBs decoder in Rust.

Install `ckb-vm-runner`:
```sh
$ git clone https://github.com/nervosnetwork/ckb-vm
$ cargo install --path . --example ckb-vm-runner
```

For quick run:

```sh
$ cargo run -- ac7b88 "[[\"Name\",\"string\",0,1,\"options\",[\"Alice\",\"Bob\",\"Charlie\",\"David\",\"Ethan\",\"Florence\",\"Grace\",\"Helen\"]],[\"Age\",\"number\",1,1,\"range\",[0,100]],[\"Score\",\"number\",2,1,\"raw\"]]"

or

$ cargo build --release
$ ckb-vm-runner target/riscv64imac-unknown-none-elf/release/spore-dobs-decoder ac7b88 "[[\"Name\",\"string\",0,1,\"options\",[\"Alice\",\"Bob\",\"Charlie\",\"David\",\"Ethan\",\"Florence\",\"Grace\",\"Helen\"]],[\"Age\",\"number\",1,1,\"range\",[0,100]],[\"Score\",\"number\",2,1,\"raw\"]]"


"[{\"name\":\"Name\",\"traits\":[{\"String\":\"Ethan\"}]},{\"name\":\"Age\",\"traits\":[{\"Number\":23}]},{\"name\":\"Score\",\"traits\":[{\"Number\":136}]}]"
```

How to integrate:
1. install `ckb-vm-runner` into your back server natively
2. call `ckb-vm-runner` with the path of `spore-dobs-decoder` binary, DNA and Pattern parameters in your server code (refer to above quick run)
3. parse the JSON traits result
