# RISC-V Trace Decoder

This is a risc-v trace decoder implementation in rust. 
For now, it only target baremetal traces in the N-trace (nexus trace) format. 

```
cargo run -- -e <elf> -t <trace> <-v>
```

For generating the encoded trace or check the encoder implementation, see:
https://github.com/iansseijelly/riscv-isa-sim/tree/n_trace. 