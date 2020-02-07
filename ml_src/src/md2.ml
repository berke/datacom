let fp = Printf.fprintf
let pf = Printf.printf

let fiota n f =
  let rec loop i =
    if i = n then
      []
    else
      f i :: loop (i + 1)
  in
  if n > 0 then
    loop 0
  else
    []

let iota n =
  let rec loop i =
    if i = n then
      []
    else
      i :: loop (i + 1)
  in
  loop 0

let pi_subst_table = [|
  41; 46; 67; 201; 162; 216; 124; 1; 61; 54; 84; 161; 236; 240; 6;
  19; 98; 167; 5; 243; 192; 199; 115; 140; 152; 147; 43; 217; 188;
  76; 130; 202; 30; 155; 87; 60; 253; 212; 224; 22; 103; 66; 111; 24;
  138; 23; 229; 18; 190; 78; 196; 214; 218; 158; 222; 73; 160; 251;
  245; 142; 187; 47; 238; 122; 169; 104; 121; 145; 21; 178; 7; 63;
  148; 194; 16; 137; 11; 34; 95; 33; 128; 127; 93; 154; 90; 144; 50;
  39; 53; 62; 204; 231; 191; 247; 151; 3; 255; 25; 48; 179; 72; 165;
  181; 209; 215; 94; 146; 42; 172; 86; 170; 198; 79; 184; 56; 210;
  150; 164; 125; 182; 118; 252; 107; 226; 156; 116; 4; 241; 69; 157;
  112; 89; 100; 113; 135; 32; 134; 91; 207; 101; 230; 45; 168; 2; 27;
  96; 37; 173; 174; 176; 185; 246; 28; 70; 97; 105; 52; 64; 126; 15;
  85; 71; 163; 35; 221; 81; 175; 58; 195; 92; 249; 206; 186; 197;
  234; 38; 44; 83; 13; 110; 133; 40; 132; 9; 211; 223; 205; 244; 65;
  129; 77; 82; 106; 220; 55; 200; 108; 193; 171; 250; 36; 225; 123;
  8; 12; 189; 177; 74; 120; 136; 149; 139; 227; 99; 232; 109; 233;
  203; 213; 254; 59; 0; 29; 57; 242; 239; 183; 14; 102; 88; 208; 228;
  166; 119; 114; 248; 235; 117; 75; 10; 49; 68; 80; 180; 143; 237;
  31; 26; 219; 153; 141; 51; 159; 17; 131; 20
|]

let weight x =
  let x = (x land 0x55) + ((x lsr 1) land 0x55) in
  let x = (x land 0x33) + ((x lsr 2) land 0x33) in
  let x = (x land 0x0f) + ((x lsr 4) land 0x0f) in
  x

let weights = Array.init 256 weight

let inv_weights =
  let a = Array.make 9 [] in
  for i = 0 to 255 do
    let w = weights.(i) in
    a.(w) <- i :: a.(w)
  done;
  a

let measure_differential f w =
  let a = Array.of_list inv_weights.(w) in
  let m = Array.length a in
  let h = Array.init m (fun _ -> Array.make 256 0) in
  for i = 0 to m - 1 do
    let delta = a.(i) in
    for j = 0 to 255 do
      let x = f j
      and y = f (j lxor delta)
      in
      let d = x lxor y in
      h.(i).(d) <- h.(i).(d) + 1;
    done
  done;
  (a, h)

let show_differential ?(threshold=0) (a, h) =
  let m = Array.length a in
  let b = Array.make (m * 256) (0, 0, 0) in
  for i = 0 to m - 1 do
    for j = 0 to 255 do
      b.(i * 256 + j) <- (a.(i), h.(i).(j), j)
    done
  done;
  Array.sort (fun (_,p,_) (_,p',_) -> - compare p p') b;
  Array.iter (fun (dx, p, dy) ->
    if p > threshold then
      pf "x^y=0x%02x =(%d)=> f(x)^f(y)=0x%02x\n"
        dx p dy) b

let pi i = pi_subst_table.(i)

module F = struct
  let show f =
    pf "{";
    for i = 0 to 255 do
      pf " %d->%d" i (f i)
    done;
    pf " }"

  let compose f g i = f (g i)

  let power n f i =
    let rec loop n f i =
      if n = 0 then
        i
      else
        if n land 1 = 0 then
          loop (n lsr 1) (compose f f) i
        else
          loop (n lsr 1) (compose f f) (f i)
    in
    loop n f i
    
  module IS = Set.Make(struct
    type t = int
    let compare (x : int) y = compare x y
  end)

  let cycles f =
    let q = ref IS.empty in
    for i = 0 to 255 do
      q := IS.add i !q
    done;
    let c = ref [] in
    while not (IS.is_empty !q) do
      let i = IS.min_elt !q in
      let rec loop s seq j =
        if IS.mem j s then
          c := (List.rev seq) :: !c
        else begin
          q := IS.remove j !q;
          loop (IS.add j s) (j :: seq) (f j)
        end
      in
      loop IS.empty [] i
    done;
    List.rev !c
end

module type OPS = sig
  type t
  val zero : t
  val const : int -> t
  val add : t -> t -> t
  val xor : t -> t -> t
  val pi : t -> t
end

module Concrete = struct
  type t = int
  let zero = 0
  let const i = i
  let add x y = (x + y) land 255
  let xor x y = x lxor y
  let pi x = pi_subst_table.(x)
end

module Terms = struct
  type t =
  | Input of int (* n-th input byte *)
  | Pi of t (* i-th entry of substitution table *)
  | Const of int (* constant byte *)
  | Add of t * t (* sum of ts *)
  | Xor of t * t

  let zero = Const 0

  let const i = Const i

  let add t1 t2 =
    match t1, t2 with
    | Const k1, Const k2 -> Const((k1 + k2) land 255)
    | _ -> Add(t1, t2)

  let xor t1 t2 =
    match t1, t2 with
    | Const k1, Const k2 -> Const((k1 lxor k2) land 255)
    | _ ->
        if t1 = t2 then
          Const 0
        else
          Xor(t1, t2)

  let pi t =
    match t with
    | Const k -> Const(pi_subst_table.(k))
    | _ -> Pi t

  let rec random_term n =
    match Random.int 5 with
    | 0 -> Input(Random.int n)
    | 1 -> Pi(random_term n)
    | 2 -> Const(Random.int 256)
    | 3 ->
        begin match Random.int 2 with
        | 0 -> Add(random_term n, random_term n)
        | _ -> let t = random_term n in Add(t, t)
        end
    | _ ->
        begin match Random.int 2 with
        | 0 -> Xor(random_term n, random_term n)
        | _ -> let t = random_term n in Xor(t, t)
        end

  let rec size t =
    match t with
    | Xor(t1, t2)|Add(t1, t2) -> 1 + (size t1) + (size t2)
    | Pi t -> 1 + size t
    | _ -> 1

  let rec print oc = function
    | Xor(t1, t2) -> fp oc "Xor(%a,%a)" print t1 print t2
    | Add(t1, t2) -> fp oc "Add(%a,%a)" print t1 print t2
    | Const i -> fp oc "Const(%d)" i
    | Pi t -> fp oc "Pi(%a)" print t
    | Input i -> fp oc "Input(%d)" i

  let rec eval env t =
    let f = eval env in
    match t with
    | Xor(t1, t2) -> (f t1) lxor (f t2)
    | Add(t1, t2) -> ((f t1) + (f t2)) land 255
    | Const i -> i
    | Pi t -> pi_subst_table.(f t)
    | Input i -> env i

  let make_input m = Array.init m (fun i -> Input i)
end

module type ROUNDS = sig
  val outer_rounds : int
  val inner_rounds : int
end

module Short_rounds = struct
  let outer_rounds = 2
  let inner_rounds = 48
end

module Full_rounds = struct
  let outer_rounds = 18
  let inner_rounds = 48
end

module MD2(R : ROUNDS)(O : OPS) = struct
  open R
  open O

  type q = {
    state : t array;
    checksum : t array;
    buffer : t array;
    mutable count : int;
  }

  let init () =
    {
      state = Array.make 16 zero;
      checksum = Array.make 16 zero;
      buffer = Array.make 16 zero;
      count = 0;
    }

  let make_padding n = Array.init n (fun i -> const n)

  let get_padding = make_padding

  let transform q b =
    let x = Array.make 48 zero in
    Array.blit q.state 0 x 0 16;
    Array.blit b 0 x 16 16;
    for i = 0 to 15 do
      x.(i + 32) <- xor q.state.(i) b.(i)
    done;
    let rec loop0 i t =
      if i < outer_rounds then
        let rec loop1 j t =
          if j < inner_rounds then begin
            let t = xor (pi t) x.(j) in
            x.(j) <- t;
            loop1 (j + 1) t
          end else
            loop0 (i + 1) (add t (const i))
        in
        loop1 0 t
      else
        ()
    in
    loop0 0 zero;
    Array.blit x 0 q.state 0 16;
    let rec cloop0 i t =
      if i < 16 then begin
        let t = pi (xor b.(i) t) in
        let t = xor q.checksum.(i) t in
        q.checksum.(i) <- t;
        cloop0 (i + 1) t
      end else
        ()
    in
    cloop0 0 q.checksum.(15)

  let update q u m =
    let index = q.count in
    q.count <- (index + m) land 15;
    let part_len = 16 - index in
    let index, i =
      if m >= part_len then begin
        Array.blit u 0 q.buffer index part_len;
        transform q q.buffer;
        let rec loop i =
          if i + 15 < m then begin
            transform q (Array.sub u i 16);
            loop (i + 16)
          end else
            0, i
        in
        loop part_len;
      end else
        index, 0
    in
    Array.blit u i q.buffer index (m - i)

  let final q =
    let index = q.count in
    let pad_len = 16 - index in
    update q (get_padding pad_len) pad_len;
    update q q.checksum 16;
    Array.copy q.state

  let array_of_string u =
    Array.init (String.length u) (fun i -> Char.code u.[i])

  let digest u =
    let q = init () in
    update q u (Array.length u);
    final q
end

module Mexpr = struct
  open Terms

  type 'a mexpr =
    | XOR of 'a * 'a
    | ADD of 'a * 'a
    | CONST of int
    | PI of 'a
    | INPUT of int

  let print_mexpr oc = function
    | XOR(i,j) -> fp oc "XOR(%d, %d);\n" i j
    | ADD(i,j) -> fp oc "ADD(%d, %d);\n" i j
    | CONST i -> fp oc "CONST %d;\n" i
    | PI i -> fp oc "PI %d;\n" i
    | INPUT i -> fp oc "INPUT %d;\n" i

  let linearize ta =
    let tbl = Hashtbl.create 1000 in
    let instr = Queue.create () in
    let loc = ref 0 in
    let rec emit t =
      try
        Hashtbl.find tbl t
      with
      | Not_found ->
          let m =
            match t with
            | Input i -> INPUT i
            | Const i -> CONST i
            | Pi t -> PI (emit t)
            | Xor(t1, t2) -> XOR(emit t1, emit t2)
            | Add(t1, t2) -> ADD(emit t1, emit t2)
          in
          let l = !loc in
          Hashtbl.add tbl t l;
          Queue.push m instr;
          incr loc;
          l
    in
    let m = Array.length ta in
    let ha = Array.init m (fun i -> emit ta.(i)) in
    let n = Queue.length instr in
    let i = ref 0 in
    let a = Array.make n (CONST 0) in
    Queue.iter (fun x -> a.(!i) <- x; incr i) instr;
    (a, ha)

  let eval_mexpr input a =
    let m = Array.length a in
    let b = Array.make m 0 in
    for i = 0 to m - 1 do
      b.(i) <-
        match a.(i) with
          | PI j -> pi_subst_table.(b.(j))
          | XOR(j, k) -> b.(j) lxor b.(k)
          | ADD(j, k) -> (b.(j) + b.(k)) land 255
          | CONST k -> k
          | INPUT j -> input j
    done;
    b

  let eval_term input t =
    let a, ha = linearize [|t|] in
    let b = eval_mexpr input a in
    b.(ha.(0))

  let test_eval_1 t =
    let t_l, h_l = linearize [| t |] in
    let k = Array.length t_l - 1 in
    for j = 0 to 255 do
      let e i = j in
      let x0 = eval e t
      and x1 = (eval_mexpr e t_l).(k)
      in 
      if x0 <> x1 then
        Printf.printf "%d %d <> %d\n" j x0 x1
    done

  let test_eval n t count =
    let t_l, h_l = linearize [| t |] in
    let args = Array.make n 0 in
    for j = 1 to count do
      for k = 0 to n - 1 do
        args.(k) <- Random.int 256
      done;
      let e i = args.(i) in
      let x0 = eval e t
      and x1 = (eval_mexpr e t_l).(h_l.(0))
      in 
      if x0 <> x1 then
        Printf.printf "%d %d <> %d\n" j x0 x1
    done

  let test count1 count2 =
    for i = 1 to count1 do
      let n = 1 + Random.int 10 in
      let t = random_term n in
      test_eval n t count2
    done
end

module Solver = struct
  open Mexpr

  module GS = Set.Make(struct
    type t = int
    let compare (x : int) y = compare x y
  end)

  module IM = Map.Make(struct
    type t = int
    let compare (x : t) y = compare x y
  end)

  module IIS = Set.Make(struct
    type t = int
    let compare (x : int) y = compare x y
  end)

  type solver = {
    circuit : int mexpr array;
    values : int array;
    outputs : int array array;
    constraints : int array array;
    constraint_indicators : bool array array;
    unsatisfied : int array;
    mutable num_unsatisfied : int;
    satisfied : bool array;
    output_nodes : int array;
  }

  let list_add i l =
    if List.mem i l then
      l
    else
      i :: l

  let compute_outputs circuit =
    let m = Array.length circuit in
    let outputs = Array.make m [] in
    for i = 0 to m - 1 do
      match circuit.(i) with
      | XOR(j, k) | ADD(j, k) ->
          outputs.(j) <- list_add i outputs.(j);
          outputs.(k) <- list_add i outputs.(k)
      | PI j ->
          outputs.(j) <- list_add i outputs.(j)
      | _ -> ()
    done;
    Array.map Array.of_list outputs

  let solver circuit output_nodes =
    let m = Array.length circuit in
    {
      circuit = circuit;
      values = Array.make m 0;
      outputs = compute_outputs circuit;
      constraints = Array.init m
        (fun _ -> Array.init 256 (fun i -> i));
      constraint_indicators = Array.init m (fun _ -> Array.make 256 true);
      unsatisfied = Array.init m (fun i -> i);
      num_unsatisfied = m;
      satisfied = Array.make m false;
      output_nodes = output_nodes;
    }

  let mark_unsatisfied q i =
    q.unsatisfied.(q.num_unsatisfied) <- i;
    q.num_unsatisfied <- q.num_unsatisfied + 1;
    q.satisfied.(i) <- false

  let mark_satisfied q i =
    q.satisfied.(i) <- true;
    q.num_unsatisfied <- q.num_unsatisfied - 1;
    q.unsatisfied.(i) <- q.unsatisfied.(q.num_unsatisfied)

  let unsatisfy q i = if q.satisfied.(i) then mark_unsatisfied q i

  let set q i x =
    unsatisfy q i;
    q.values.(i) <- x

  let is_gate_constraint_satisfied q i =
    q.constraint_indicators.(i).(q.values.(i))

  let is_gate_satisfied q i =
    is_gate_constraint_satisfied q i &&
    match q.circuit.(i) with
    | XOR(j, k) -> q.values.(i) = q.values.(j) lxor q.values.(k)
    | ADD(j, k) ->
        q.values.(i) = 255 land (q.values.(j) + q.values.(k))
    | CONST k -> q.values.(i) = k
    | PI j -> q.values.(i) = pi_subst_table.(q.values.(j))
    | INPUT i -> true

  let check q i =
    if is_gate_satisfied q i then
      if not q.satisfied.(i) then
        mark_satisfied q i
      else
        ()
    else
      if q.satisfied.(i) then
        mark_unsatisfied q i
      else
        ()

  let propagate q i =
    match q.circuit.(i) with
    | XOR(j, k) -> set q i (q.values.(j) lxor q.values.(k))
    | ADD(j, k) -> set q i (255 land (q.values.(j) + q.values.(k)))
    | PI j -> set q i (pi_subst_table.(q.values.(j)))
    | CONST k -> set q i k
    | INPUT _ -> ()

  let set_constraint q i a =
    q.constraints.(i) <- a;
    for j = 0 to 255 do
      q.constraint_indicators.(i).(j) <- false;
    done;
    Array.iter (fun j -> q.constraint_indicators.(i).(j) <- true) a

  let inverse_pi =
    let a = Array.make 256 0 in
    for i = 0 to 255 do
      a.(pi_subst_table.(i)) <- i
    done;
    a

  let pick a = a.(Random.int (Array.length a))

  let satisfy_backward q i =
    if not q.constraint_indicators.(i).(q.values.(i)) then
      q.values.(i) <- pick q.constraints.(i);
    match q.circuit.(i) with
    | CONST k -> ()
    | PI j ->
        set q j inverse_pi.(q.values.(i))
    | ADD(j, k) ->
        let x = pick q.constraints.(j) in
        set q j x;
        set q k ((q.values.(i) - x) land 255)
    | XOR(j, k) ->
        let x = pick q.constraints.(j) in
        set q j x;
        set q k (q.values.(i) lxor x)
    | INPUT _ -> ()

  let randomize q i =
    set q i (pick q.constraints.(i))

  let unsatisfied q =
    let m = Array.length q.circuit in
    let a = Array.make m 0 in
    let rec loop i j =
      if i = m then
        Array.sub a 0 j
      else
        if is_gate_satisfied q i then
          loop (i + 1) j
        else
          begin
            a.(j) <- i;
            loop (i + 1) (j + 1)
          end
    in
    loop 0 0

  let count_unsatisfied q =
    let m = Array.length q.circuit in
    let rec loop n i =
      if i = m then
        n
      else
        if q.satisfied.(i) then
          loop n (i + 1)
        else
          loop (n + 1) (i + 1)
    in
    loop 0 0

  let p_randomize = 64
  let p_forward = 256

  let dump oc q =
    let m = Array.length q.circuit in
    fp oc "{";
    for i = 0 to m - 1 do
      fp oc " %d:%d" i q.values.(i)
    done;
    fp oc " }"

  let iteration q =
    (*pf "iter %a\n%!" dump q;*)
    (* Pick a random unsatisfied node *)
    if q.num_unsatisfied = 0 then
      false
    else
      begin
        assert (count_unsatisfied q = q.num_unsatisfied);

        let p = Random.int 1000 in
        if p < p_randomize then
          begin
            let i = Random.int (Array.length q.circuit) in
            randomize q i;
            (*pf "randomize %d <- %d\n" i q.values.(i)*)
          end;

        let j = Random.int q.num_unsatisfied in
        let i = q.unsatisfied.(j) in
        propagate q i;
        (*input_line stdin;*)
        if is_gate_satisfied q i then
          () (*pf "ok %d (%d)\n%!" i q.values.(i)*)
        else
          begin
            (*pf "sat back %d\n%!" i;*)
            satisfy_backward q i
          end;
        (*else if p < p_forward then
          begin
            propagate q i
          end
        else
          begin
            satisfy_backward q i
          end;*)
        (*if is_gate_constraint_satisfied q i then*)
        check q i;
        (* Check if outputs became unsatisfied *)
        if false then
          pf "unsat %d/%d pick %d -> %d\n"
            q.num_unsatisfied
            (Array.length q.circuit)
            i
            (Array.length q.outputs.(i));
        Array.iter (check q) q.outputs.(i);
        true
      end

  exception Double_input of int

  let collect_inputs q =
    let m = Array.length q.circuit in
    let rec loop i map n =
      if i = m then
        Array.init n (fun j -> IM.find j map)
      else
        match q.circuit.(i) with
        | INPUT j ->
            if IM.mem j map then
              raise (Double_input j)
            else
              loop (i + 1) (IM.add j i map) (n + 1)
        | _ -> loop (i + 1) map n
    in
    loop 0 IM.empty 0

  let run q =
    let m = Array.length q.circuit in
    let rec loop i =
      if i land 4095 = 0 then
        pf "Iteration: %d (%d/%d unsatisfied)\n%!"
          i q.num_unsatisfied m;
      if iteration q then
        loop (i + 1)
      else
        begin
          pf "Done!\n";
          pf "Sanity check...\n";
          let a = unsatisfied q in
          let n = Array.length a in
          if n = 0 then
            begin
              pf "Circuit state:\n";
              dump stdout q;
              pf "Input:";
              let x = collect_inputs q in
              for i = 0 to Array.length x - 1 do
                pf " %02x" q.values.(x.(i))
              done;
              pf "\n";
              pf "Output:";
              for i = 0 to Array.length q.output_nodes - 1 do
                pf " %02x" q.values.(q.output_nodes.(i))
              done;
              pf "\n"
            end
          else
            begin
              pf "Failure.  Unsatisfied gates:";
              for i = 0 to n - 1 do
                pf " %d" a.(i)
              done;
              pf "\n"
            end
        end
    in
    loop 0
end

module R = Short_rounds
module MD2_T = MD2(R)(Terms)
module MD2_C = MD2(R)(Concrete)

module Test(R : ROUNDS) = struct
  let n = 4

  let x = Terms.make_input n

  let d0 = MD2_T.digest x

  let d0_l, h_l = Mexpr.linearize d0

  let d0_e = Mexpr.eval_mexpr (fun i -> 0) d0_l

  let d1 = MD2_C.digest (Array.make n 0)
end

let hex_of_array a =
  let m = Array.length a in
  let b = Buffer.create (2 * m) in
  for i = 0 to m - 1 do
    Printf.bprintf b "%02x" a.(i)
  done;
  Buffer.contents b

let pf = Printf.printf

let print_pi () =
  let a = Array.make 256 [] in
  for i = 0 to 255 do
    let j = pi_subst_table.(i) in
    a.(j) <- i :: a.(j)
  done;
  for j = 0 to 255 do
    pf "{";
    List.iter (fun i -> pf " %d" i) a.(j);
    pf " } -> %d\n" j
  done

let pi_xor_cycle_lengths () =
  List.map
    (fun j -> List.map List.length (F.cycles (fun k -> pi k lxor j)))
    (iota 256);;

module BO = struct
  open Big_int

  let b0 = zero_big_int
  let b1 = unit_big_int
  let ( ++ ) = add_big_int
  let ( -- ) = sub_big_int
  let ( ** ) = mult_big_int
  let ( // ) = div_big_int
end

open Big_int
open BO

let lcm_big_int p q = (p ** q) // (gcd_big_int p q)

let lcm_of_list l =
  List.fold_left
    (fun m n -> lcm_big_int m n) unit_big_int
    l

let list_iteri f =
  let rec loop i = function
    | [] -> ()
    | l :: r ->
        f i l;
        loop (i + 1) r
  in
  loop 0

let cycrep c =
  let a = Array.make 256 (-1,-1,-1) in
  list_iteri
    begin fun i l ->
      let m = List.length l in
      list_iteri begin fun j x ->
        a.(x) <- (i, j, m)
      end l
    end
    c;
  a

let pi_cycles = F.cycles pi
let pi_cycles_a = Array.of_list (List.map Array.of_list (pi_cycles))
let pi_cycrep = cycrep pi_cycles

let solve f =
  let rec loop i =
    if i < 256 then
      if f i = 0 then
        i
      else
        loop (i + 1)
    else
      raise Not_found
  in
  loop 0

module T = Test(Full_rounds)

let array_map2 f a1 a2 =
  let m = Array.length a1 in
  let a = Array.create m (f a1.(0) a2.(0)) in
  for i = 1 to m - 1 do
    a.(i) <- f a1.(i) a2.(i)
  done;
  a

open Mexpr

let d, h =
  if true then
    [|
      (* 0 *) INPUT 0;
      (* 1 *) INPUT 1;
      (* 2 *) PI 0;
      (* 3 *) XOR(0, 2);
      (* 4 *) CONST 55;
      (* 5 *) ADD(3, 4);
    |],
    [| 5 |]
  else
    T.d0_l, T.h_l

let q = Solver.solver d h

let _ =
  if false then begin
    let k = Array.init 256 (fun i -> i) in
    for i = 0 to Array.length h - 1 do
      Solver.set_constraint q h.(i) k;
    done
  end

let _ =
  Solver.set_constraint q h.(0) [| 33; 34; 35; 36 |];
  Solver.run q

(* vim:set tw=80 ts=2 sw=2 expandtab: *)
