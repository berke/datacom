open Sexplib
open Conv

module P = Printf
module S = Scanf

module type VAR = sig
  type v
end

type ('u,'v) term =
  | Const of bool
  | Var of 'v
  | Sum of 'u * 'u
  | Mul of 'u * 'u [@deriving sexp]

module Make_Term(V : VAR) = struct
  type t = (u,V.v) term
  and u = int [@deriving sexp]
  let compare = compare
end

module Var = struct
  type v = int
end

let rec count_leading_zeroes n =
  if n land 65535 = 0 then
    16 + count_leading_zeroes (n lsr 16)
  else
    if n land 255 = 0 then
      8 + count_leading_zeroes (n lsr 8)
    else
      if n land 16 = 0 then
        4 + count_leading_zeroes (n lsr 4)
      else
        if n = 0 then
          63
        else
          if n land 1 = 0 then
            1 + count_leading_zeroes (n lsr 1)
          else
            0

module BitSet = struct
  let n = 63

  module Index = struct
    type t = int
    let compare (x : t) y = compare x y
  end

  module IM = Map.Make(Index)

  type t = int IM.t

  let empty = IM.empty

  let add i bs =
    let j = i / n in
    let k = i mod n in
    match IM.find_opt j bs with
    | None -> IM.add j (1 lsl k) bs
    | Some w -> IM.add j ((1 lsl k) lor w) bs

  let or_zero = function
    | None -> 0
    | Some w -> w

  let union bs1 bs2 =
    IM.merge
      (fun _ w1o w2o -> Some((or_zero w1o) lor (or_zero w2o)))
      bs1 bs2

  let is_empty bs = IM.for_all (fun _j w -> w = 0) bs

  let iter f bs =
    IM.iter (fun j w ->
        let rec loop k w =
          if k = n then
            ()
          else
            (
              if w land 1 <> 0 then f (n*j + k);
              loop (k + 1) (w lsr 1)
            )
        in
        loop 0 w)
    bs
end

module Make_Memo(V : VAR) = struct
  module Term = Make_Term(V)

  module TM = Map.Make(Term)

  module I = struct
    type t = Term.u
    let compare (x : Term.u) y = compare x y
  end

  module IM = Map.Make(I)

  type t = {
      mutable idx : int TM.t;
      mutable xdi : Term.t IM.t;
      mutable ctr : int;
    }

  let make () =
    {
      idx = TM.empty;
      xdi = IM.empty;
      ctr = 0
    }

  let count mm = mm.ctr
    
  let register mm t =
    match TM.find_opt t mm.idx with
    | None ->
       let c = mm.ctr in
       mm.ctr <- c + 1;
       mm.xdi <- IM.add c t mm.xdi;
       mm.idx <- TM.add t c mm.idx;
       c
    | Some c -> c

  let get mm u = IM.find u mm.xdi

  let to_array mm = Array.init mm.ctr (get mm)

  let const mm b = register mm (Const b)

  let var mm v = register mm (Var v)

  let info mm u =
    match u with
    | Var _ -> None
    | Const false -> Some false
    | Const true -> Some true
    | _ -> None

  let sum mm u1 u2 =
    match get mm u1, get mm u2 with
    | Const false, _ -> u2
    | _, Const false -> u1
    | _, _ ->
       if u1 = u2 then
         const mm false
       else
         register mm (Sum(min u1 u2,max u1 u2))

  let mul mm u1 u2 =
    match get mm u1,get mm u2 with
    | Const false, _ | _, Const false -> const mm false
    | Const true, _ -> u2
    | _, Const true -> u1
    | _, _ ->
       if u1 = u2 then
         u1
       else
         register mm (Mul(min u1 u2,max u1 u2))
end

module Memo = Make_Memo(Var)

exception Loop of int
exception Not_reached of int

let cse a =
  let m = Array.length a in
  let mm = Memo.make () in
  let b = Array.make m `Undefined in
  let rec convert i =
    match b.(i) with
    | `Defined u -> u
    | `Busy -> raise (Loop i)
    | `Undefined ->
       (
         b.(i) <- `Busy;
         let u =
           match a.(i) with
           | Var v -> Memo.var mm v
           | Const b -> Memo.const mm b
           | Sum(i,j) -> Memo.sum mm (convert i) (convert j)
           | Mul(i,j) -> Memo.mul mm (convert i) (convert j)
         in
         b.(i) <- `Defined u;
         u
       )
  in
  let tr = Array.make m 0 in
  for i = 0 to m - 1 do
    tr.(i) <- convert i
  done;
  (Memo.to_array mm,tr)

module type MORPHISM = sig
  type state
  type q
  val const : state -> bool -> q
  val var : state -> int -> q
  val sum : state -> q -> q -> q
  val mul : state -> q -> q -> q
end

(* let linearize a =
 *   let m = Array.length a in *)
  

module Morphism_Evaluator(M : MORPHISM) = struct
  let eval state a =
    let m = Array.length a in
    let mm = Memo.make () in
    let b = Array.make m `Undefined in
    let rec compute i =
      match b.(i) with
      | `Defined u -> u
      | `Busy -> raise (Loop i)
      | `Undefined ->
         (
           b.(i) <- `Busy;
           let u =
             match a.(i) with
             | Var v -> M.var state v
             | Const b -> M.const state b
             | Sum(i,j) -> M.sum state (compute i) (compute j)
             | Mul(i,j) -> M.mul state (compute i) (compute j)
           in
           b.(i) <- `Defined u;
           u
         )
    in
    Array.init m compute
end

module Variable_Set_Morphism = struct
  type state = ()
  type q = BitSet.t

  let const () _ = BitSet.empty

  let var () i = BitSet.add i BitSet.empty

  let sum () q1 q2 = BitSet.union q1 q2

  let mul () q1 q2 = BitSet.union q1 q2
end

module VSME = struct
  include Morphism_Evaluator(Variable_Set_Morphism)

  let dump vs oc =
    for i = 0 to Array.length vs - 1 do
      P.fprintf oc "%d : {" i;
      BitSet.iter (fun j -> P.fprintf oc " %d" j) vs.(i);
      P.fprintf oc " }\n"
    done
end

let dump_with_var dump_var a oc =
  let m = Array.length a in
  for i = 0 to m - 1 do
    P.fprintf oc "%d " i;
    match a.(i) with
    | Var v -> P.fprintf oc "V %a\n" dump_var v
    | Sum(i,j) -> P.fprintf oc "S %d %d\n" i j
    | Mul(i,j) -> P.fprintf oc "M %d %d\n" i j
    | Const false -> P.fprintf oc "C 0\n"
    | Const true -> P.fprintf oc "C 1\n"
  done

let dump = dump_with_var (fun oc v -> P.fprintf oc "%d" v)

module QuadVar = struct
  type v =
    | Lin of int
    | Quad of int * int
end

module QuadTerm = Make_Term(QuadVar)

module Linearization_Morphism = struct
  module Memo = Make_Memo(QuadVar)

  type state = Memo.t

  type q = Memo.Term.u

  let const mm b = Memo.const mm b

  let var mm v = Memo.var mm (QuadVar.Lin v)

  let sum mm t1 t2 = Memo.sum mm t1 t2

  let mul mm t1 t2 =
    if t1 = t2 then
      t1
    else
      Memo.var mm (QuadVar.Quad(min t1 t2,max t1 t2))
end

module LME = struct
  include Morphism_Evaluator(Linearization_Morphism)

  let dump (a,mm) oc =
    dump_with_var (fun oc v ->
        match v with
        | QuadVar.Lin v -> P.fprintf oc "v%d" v
        | QuadVar.Quad(t1,t2) -> P.fprintf oc "t(%d,%d)" t1 t2)
    a oc
end



let load fn =
  let ic = open_in fn in
  let sic = S.Scanning.from_channel ic in
  let m = S.bscanf sic "%d" (fun m -> m) in
  P.eprintf "Terms: %d\n%!" m;
  let a = Array.make m (Const false) in
  let g () = S.bscanf sic " %d" (fun x -> x) in
  for i = 0 to m - 1 do
    a.(i) <-
      match S.bscanf sic "\n%c" (fun c -> c) with
      | 'C' -> Const(g () = 1)
      | 'V' -> Var(g ())
      | 'A' ->
         let i = g () in
         let j = g () in
         Sum(i,j)
      | 'M' ->
         let i = g () in
         let j = g () in
         Mul(i,j)
      | c -> failwith (P.sprintf "Invalid character %C in term %d" c i)
  done;
  a

let wrap x g f =
  try
    let y = f x in
    g x;
    y
  with
  | e ->
     g x;
     raise e
  
let _ =
  let a = load Sys.argv.(1) in
  let m = Array.length a in
  P.printf "Initial: %d\n%!" m;
  let (a',tr) = cse a in
  let m' = Array.length a' in
  P.printf "Optimized: %d\n%!" m';
  wrap (open_out "opt.gt") close_out (dump a');
  wrap (open_out "opt.tr") close_out (fun oc -> Array.iteri (fun i u -> P.fprintf oc "%d %d\n" i u) tr);
  let vs = VSME.eval () a' in
  wrap (open_out "opt.vs") close_out (VSME.dump vs);
  let mm = Linearization_Morphism.Memo.make () in
  let b' = LME.eval mm a' in
  let b' = Linearization_Morphism.Memo.to_array mm in
  P.printf "Linearized: %d\n%!" (Array.length b');
  wrap (open_out "opt.lin") close_out (LME.dump (b',mm))
