From Stdlib Require Import String.
Require Import Namu.terms.

Open Scope string_scope.
Open Scope tree_scope.

(* Serialize an arbitrary Tree (including applications) as space-separated
   ternary-encoded programs.  For a left-associated application chain
   f @ a1 @ a2 @ ... @ aN, produces "ternary(f) ternary(a1) ternary(a2) ... ternary(aN)".
   The Rust side splits on spaces and folds with apply. *)
Fixpoint apps_to_ternary (t : Tree) : string :=
  match t with
  | Ref _ => "fail"
  | Node => "0"
  | Node @ p1 => "1" ++ program_to_ternary p1
  | Node @ p1 @ p2 => "2" ++ program_to_ternary p1 ++ program_to_ternary p2
  | m @ n => apps_to_ternary m ++ " " ++ program_to_ternary n
  end.

Definition labeled (name : string) (input expected : Tree) : string :=
  name ++ " ||| " ++ apps_to_ternary input ++ " ||| " ++ apps_to_ternary expected.

(* Triage calculus rules -------------------------------------------------------- *)

(* k_red: △ △ y z -> y *)
Eval compute in labeled "k_leaf_leaf"   (K @ △ @ △) (△).
Eval compute in labeled "k_K_leaf"      (K @ K @ △) (K).
Eval compute in labeled "k_fork_K"      (K @ (△@△@△) @ K) (△@△@△).

(* s_red: △ (△ x) y z -> x z (y z) *)
(* (xz)(yz) in triage matches arbre *)
(* S K K x = I x = x *)
Eval compute in labeled "i_leaf"        (I @ △) (△).
Eval compute in labeled "i_K"           (I @ K) (K).
Eval compute in labeled "i_KI"          (I @ KI) (KI).

(* Triage: leaf_red, stem_red, fork_red *)
(* triage f0 f1 f2 △ -> f0 *)
Eval compute in labeled "triage_leaf"   (triage K I △ @ △) (K).
(* triage f0 f1 f2 (△ z) → f1 z *)
Eval compute in labeled "triage_stem"   (triage K I △ @ (△ @ △)) (I @ △).
(* triage f0 f1 f2 (△ z1 z2) → f2 z1 z2 *)
Eval compute in labeled "triage_fork"   (triage K I △ @ (△ @ △ @ K)) (△ @ △ @ K).

(* Booleans: tt = K, ff = KI --------------------------------------------------- *)
Eval compute in labeled "tt_leaf_K"     (tt @ △ @ K) (△).
Eval compute in labeled "tt_K_leaf"     (tt @ K @ △) (K).
Eval compute in labeled "ff_leaf_K"     (ff @ △ @ K) (K).
Eval compute in labeled "ff_K_leaf"     (ff @ K @ △) (△).

(* Wait and swap --------------------------------------------------------------- *)

(* wait_red: wait M N @ x → M @ N @ x *)
Eval compute in labeled "wait_red"      (wait K △ @ I) (K @ △ @ I).

(* swap_red: swap f @ x @ y → f @ y @ x *)
Eval compute in labeled "swap_red"      (swap K @ △ @ I) (K @ I @ △).


(* Pairs ----------------------------------------------------------------------- *)

(* fstL_red: fstL @ (pairL x y) → x *)
Eval compute in labeled "fst_leaf_K"    (fstL @ (pairL △ K)) (△).
Eval compute in labeled "fst_K_leaf"    (fstL @ (pairL K △)) (K).

(* sndL_red: sndL @ (pairL x y) → y *)
Eval compute in labeled "snd_leaf_K"    (sndL @ (pairL △ K)) (K).
Eval compute in labeled "snd_K_leaf"    (sndL @ (pairL K △)) (△).

(* Equality -------------------------------------------------------------------- *)

(* equal @ leaf @ leaf → K *)
Eval compute in labeled "equal_leaf_leaf" (equal @ △ @ △) (K).

(* equal @ (stem x) @ (stem y) → equal @ x @ y *)
(* test with x=y=leaf *)
Eval compute in labeled "equal_stem_stem_leaf" (equal @ (△ @ △) @ (△ @ △)) (equal @ △ @ △).

(* equal on unequal structures *)
Eval compute in labeled "equal_leaf_stem"  (equal @ △ @ (△ @ △)) (KI).
Eval compute in labeled "equal_leaf_fork"  (equal @ △ @ (△ @ △ @ △)) (KI).
Eval compute in labeled "equal_stem_leaf"  (equal @ (△ @ △) @ △) (KI).
Eval compute in labeled "equal_stem_fork"  (equal @ (△ @ △) @ (△ @ △ @ △)) (KI).
Eval compute in labeled "equal_fork_leaf"  (equal @ (△ @ △ @ △) @ △) (KI).
Eval compute in labeled "equal_fork_stem"  (equal @ (△ @ △ @ △) @ (△ @ △)) (KI).

(* equal @ K @ K → K (reflexivity on program K) *)
Eval compute in labeled "equal_K_K"        (equal @ K @ K) (K).

(* Natural numbers: Church-like zero = KI, succ1 ------------------------------ *)
Eval compute in labeled "isZero_zero"      (isZero @ zero) (tt).
Eval compute in labeled "isZero_succ_zero" (isZero @ (succ1 @ zero)) (ff).

(* cond (= I) on booleans *)
Eval compute in labeled "cond_true"        (cond @ tt @ △ @ K) (△).
Eval compute in labeled "cond_false"       (cond @ ff @ △ @ K) (K).

(* Branch-first evaluator ----------------------------------------------------- *)

(* bf on programs *)
Eval compute in labeled "bf_leaf"          (bf @ △ @ K) (△ @ K).
Eval compute in labeled "bf_stem"          (bf @ (△ @ △) @ K) (△ @ △ @ K).
Eval compute in labeled "bf_fork_leaf"     (bf @ (△@△@K) @ I) (K).

(* Mirror test ---------------------------------------------------------------- *)

(* Print mirror's ternary string and size — read from compile output *)
Eval cbv in program_to_ternary mirror.
Eval cbv in term_size mirror.

(* mirror(leaf) = KI (false) — leaf is not mirror *)
Eval compute in labeled "mirror_leaf" (mirror @ Node) (KI).

(* mirror(K) = KI (false) — K is not mirror *)
Eval compute in labeled "mirror_K" (mirror @ K) (KI).
