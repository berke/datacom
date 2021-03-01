Cryptographic experiments on block ciphers using SAT solvers - draft code

Auther: Berké DURAK <bd@exhrd.fr>

Copyright(C) 2019-2021 except where indicated otherwise

OCaml code:
- ml/kschedeq.ml : AES key schedule locator generator.  Can be used to
recover BitLocker keys from RAM dumps when, for some reason, aeskeyfind
doesn't work.
- ml/md2.ml : MD2 pre-image constraint solver attempt

Rust code:
- test_machine.rs: Uses the cryptominisat SAT solver and its
cryptominisat-rs bindings to search for encryption keys, given
traffic.

Models are included for:

- DES (not fully verified)
- GSM A5/2 (not fully verified)
- Xtea : Verified, matches C version
- Tea : Verified, matches C version

Other programs include:

- test_distinguisher.rs: Strict Avalanche Criterion distinguisher for
reduced-round Tea based on the papers by Julio C. Hernandez and Pedro
Isasi
- test_revealer.rs: Attempt to find key revealing bits using SAT, again
for reduced rounds Tea, with a brute-force pre-filter.
- main.rs: A time memory trade-off experiment.