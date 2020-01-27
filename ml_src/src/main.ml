open Sexplib
open Conv

module P = Printf
module S = Scanf

module Term = struct
  type t =
    | Const of bool
    | Var of int
    | Sum of u * u
    | Mul of u * u
  and u = int [@deriving sexp]
  let compare = compare
end

module TM = Map.Make(Term)

module I = struct
  type t = Term.u
  let compare (x : Term.u) y = compare x y
end

module IM = Map.Make(I)

module Memo = struct

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
    | Term.Var _ -> None
    | Term.Const false -> Some false
    | Term.Const true -> Some true
    | _ -> None

  let sum mm u1 u2 =
    match get mm u1, get mm u2 with
    | Term.Const false, _ -> u2
    | _, Term.Const false -> u1
    | _, _ ->
       if u1 = u2 then
         const mm false
       else
         register mm (Term.Sum(min u1 u2,max u1 u2))

  let mul mm u1 u2 =
    match get mm u1,get mm u2 with
    | Term.Const false, _ | _, Term.Const false -> const mm false
    | Term.Const true, _ -> u2
    | _, Term.Const true -> u1
    | _, _ ->
       if u1 = u2 then
         u1
       else
         register mm (Term.Mul(min u1 u2,max u1 u2))

  (* in
   * let rec var v = get (Term.Var v)
   * and const b = get (Term.Const b)
   * and sum t1 t2 =
   *   in
   *   get (Term.Sum(t1',t2'))
   * and mul t1 t2 =
   *   let t1' = min t1 t2
   *   and t2' = max t1 t2
   *   in
   *   get (Term.Mul(t1',t2'))
   * in
   * object
   *   method var = var
   *   method const = const
   *   method sum = sum
   *   method mul = mul
   *   method count = !ctr
   * end *)
end

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
           | Term.Var v -> Memo.var mm v
           | Term.Const b -> Memo.const mm b
           | Term.Sum(i,j) -> Memo.sum mm (convert i) (convert j)
           | Term.Mul(i,j) -> Memo.mul mm (convert i) (convert j)
         in
         b.(i) <- `Defined u;
         u
       )
  in
  for i = 0 to m - 1 do
    let _ = convert i in
    ()
  done;
  Memo.to_array mm
  (* (Array.mapi (fun i t ->
   *     match t with
   *     | `Defined u -> Memo.get mm u
   *     | _ -> raise (Not_reached i))
   *   b,
   * mm) *)

let dump a oc =
  let m = Array.length a in
  let open Term in
  for i = 0 to m - 1 do
    P.fprintf oc "%d " i;
    match a.(i) with
    | Var v -> P.fprintf oc "V %d\n" v
    | Sum(i,j) -> P.fprintf oc "S %d %d\n" i j
    | Mul(i,j) -> P.fprintf oc "M %d %d\n" i j
    | Const false -> P.fprintf oc "C 0\n"
    | Const true -> P.fprintf oc "C 1\n"
  done

(* module Linearize = struct
 *   module T2M =
 *     Map.Make(
 *         struct
 *           type t = Term.t * Term.t
 *           let compare = compare
 *         end) *)

(* let linearize a =
 *   let ctr = 0 in
 *   let map = ref T2M.empty in *)
    
(* end *)

let load fn =
  let ic = open_in fn in
  let sic = S.Scanning.from_channel ic in
  let m = S.bscanf sic "%d" (fun m -> m) in
  P.eprintf "Terms: %d\n%!" m;
  let a = Array.make m (Term.Const false) in
  let g () = S.bscanf sic " %d" (fun x -> x) in
  for i = 0 to m - 1 do
    a.(i) <-
      match S.bscanf sic "\n%c" (fun c -> c) with
      | 'C' -> Term.Const(g () = 1)
      | 'V' -> Term.Var(g ())
      | 'A' ->
         let i = g () in
         let j = g () in
         Term.Sum(i,j)
      | 'M' ->
         let i = g () in
         let j = g () in
         Term.Mul(i,j)
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
  let a' = cse a in
  let m' = Array.length a' in
  P.printf "Optimized: %d\n%!" m';
  wrap (open_out "opt.gt") close_out (dump a')
