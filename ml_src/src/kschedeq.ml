open Sexplib
open Conv
open Printf
   
let z_round = [| 1;2;4;8;16;32;64;128;27;54 |]

type term =
  | C of int (* Constant *)
  | K of int (* Key byte *)
  | S of term_id (* S-box application *)
  | X of term_id * term_id (* Xor *)
and term_id = int
                [@@deriving sexp]

module TM = Map.Make(struct type t = term let compare = compare end)
module IDM = Map.Make(struct type t = term_id let compare = compare end)

let _ =
  let ctr = ref 0 in
  let map = ref TM.empty in
  let idx = ref IDM.empty in
  let gensym () =
    incr ctr;
    !ctr - 1
  in
  let get_term t =
    match TM.find_opt t !map with
    | Some(id) -> id
    | None ->
       let id = gensym () in
       map := TM.add t id !map;
       idx := IDM.add id t !idx;
       id
  in
  let sbox t = get_term (S t) in
  let xor t1 t2 =
    if t1 = t2 then
      get_term (C 0)
    else
      get_term (X(min t1 t2,max t1 t2))
  in
  let key i = get_term (K i) in
  let const c = get_term (C c) in
  let vs = Array.make (11 * 16) (-1) in (* invalid *)
  for i = 0 to 15 do
    vs.(i) <- key i;
  done;
  for rd = 1 to 10 do
    let i0 = 16 * (rd - 1) in
    let i1 = i0 + 16 in
    let e0 = xor (sbox vs.(i0 + 3*4 + 1)) (const z_round.(rd - 1)) in
    let e1 = sbox vs.(i0 + 3*4 + 2) in
    let e2 = sbox vs.(i0 + 3*4 + 3) in
    let e3 = sbox vs.(i0 + 3*4 + 0) in
    vs.(i1 + 0) <- xor e0 vs.(i0 + 0);
    vs.(i1 + 1) <- xor e1 vs.(i0 + 1);
    vs.(i1 + 2) <- xor e2 vs.(i0 + 2);
    vs.(i1 + 3) <- xor e3 vs.(i0 + 3);

    vs.(i1 + 4) <- xor vs.(i1 + 0)  vs.(i0 + 4);
    vs.(i1 + 5) <- xor vs.(i1 + 1)  vs.(i0 + 5);
    vs.(i1 + 6) <- xor vs.(i1 + 2)  vs.(i0 + 6);
    vs.(i1 + 7) <- xor vs.(i1 + 3)  vs.(i0 + 7);

    vs.(i1 + 8)  <- xor vs.(i1 + 4)  vs.(i0 + 8);
    vs.(i1 + 9)  <- xor vs.(i1 + 5)  vs.(i0 + 9);
    vs.(i1 + 10) <- xor vs.(i1 + 6)  vs.(i0 + 10);
    vs.(i1 + 11) <- xor vs.(i1 + 7)  vs.(i0 + 11);

    vs.(i1 + 12)  <- xor vs.(i1 + 8)  vs.(i0 + 12);
    vs.(i1 + 13)  <- xor vs.(i1 + 9)  vs.(i0 + 13);
    vs.(i1 + 14) <- xor vs.(i1 + 10)  vs.(i0 + 14);
    vs.(i1 + 15) <- xor vs.(i1 + 11)  vs.(i0 + 15);
  done;
  for i = 0 to Array.length vs - 1 do
    Printf.printf "%2d t%d %s\n" i vs.(i) (Sexp.to_string (sexp_of_term (IDM.find vs.(i) !idx)))
  done;
  IDM.iter (fun id t ->
      Printf.printf "t%d = %s\n" id (Sexp.to_string (sexp_of_term t)))
    !idx;
  let nt = !ctr in
  (* Generate verifier *)
  let oc = open_out "verify.c" in
  fprintf oc "#include <stdbool.h>\n\
              #include <inttypes.h>\n\
              \n\
              extern const uint8_t sbox[256];\n\
              \n\
              bool aes_verify_ks(const uint8_t *ks,int *fail_t,int *fail_k,uint8_t *expected) { \n\
             \  uint8_t t[%d]; \n\
              \n" nt;
  let evaluated = Array.make nt false in
  (* let rec verify t i =
   *   if verified.(t) then
   *     ()
   *   else
   *     ( *)
  let rec eval t =
    if not evaluated.(t) then
      (
        evaluated.(t) <- true;
        match IDM.find t !idx with
        | K k ->
           fprintf oc "  t[%d] = ks[%d^3];\n" t k
        | C c ->
           fprintf oc "  t[%d] = %d;\n" t c
        | X(t1,t2) ->
           eval t1;
           eval t2;
           fprintf oc "  t[%d] = t[%d] ^ t[%d];\n" t t1 t2;
        | S t'  ->
           eval t';
           fprintf oc "  t[%d] = sbox[t[%d]];\n" t t'
      )
  and verify t i =
    eval t;
    fprintf oc "  if (ks[%d^3] != t[%d]) { *fail_t = %d; *fail_k = %d; *expected = t[%d]; return false; }\n"
               i t t i t
  in
  for i = 0 to Array.length vs - 1 do
    verify vs.(i) i
  done;
  fprintf oc "\n\
             \  return true;\n\
              }\n";
  close_out oc
