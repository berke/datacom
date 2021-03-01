Cryptographic experiments on block ciphers using SAT solvers

Auther: Berké DURAK <bd@exhrd.fr>

Copyright(C) 2019-2021 except where indicated otherwise

OCaml code:
- ml/kschedeq.ml : AES key schedule detector generater
- ml/md2.ml : MD2 pre-image constraint solver attempt

The Rust program test_machine.rs uses the cryptominisat SAT solver and
its cryptominisat-rs bindings to search for encryption keys, given
traffic.

Models are included for:

- DES (not fully verified)
- GSM A5/2 (not fully verified)
- Xtea : Verified, matches C version
- Tea : Verified, matches C version

Other programs include:

- test_distinguisher: Strict Avalanche Criterion distinguisher for
reduced-round Tea based on the papers by Julio C. Hernandez and Pedro
Isasi
- test_revealer: Attempt to find key revealing bits using SAT, again
for reduced rounds Tea, with a brute-force pre-filter.