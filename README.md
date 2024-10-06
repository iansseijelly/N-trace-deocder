# RISC-V N-Trace Decoder

This is a risc-v n-trace decoder implementation in rust. 
For now, it only target baremetal traces in the N-trace (nexus trace) Branch-Target Mode (BTM) format. 

```
cargo run -- -e <elf> -t <trace> <-v>
```

For generating the encoded trace or check the encoder implementation, see:
https://github.com/iansseijelly/riscv-isa-sim/tree/n_trace (encoder model)
https://github.com/iansseijelly/spike-devices/tree/trace_encoder_ctrl (encoder controller)