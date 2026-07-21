//! Example: structural error folding in EDN parse errors.
//!
//! Demonstrates how [cirru_edn::parse] now folds large Cirru trees along the
//! error path, showing only relevant siblings and collapsing irrelevant branches.

use cirru_edn::parse;

fn main() {
  println!("=== EDN Error Folding Demo ===\n");

  // ----------------------------------------------------------------
  // Case 1: %{} record with malformed pair (triple instead of pair)
  // ----------------------------------------------------------------
  println!("── Case 1: %%{{}} record with a malformed (triple) pair ──\n");
  let bad_record = r#"
%{} :User
  :age 30
  (:name |Alice :extra |oops)
  :active true
"#;
  match parse(bad_record) {
    Ok(val) => println!("  Unexpected success: {val}"),
    Err(e) => println!("{e}\n"),
  }

  // ----------------------------------------------------------------
  // Case 2: Deeply nested %{} — folding siblings around the error
  // ----------------------------------------------------------------
  println!("── Case 2: nested record with error deep inside ──\n");
  let deep_nested = r#"
%{} :Package
  :name |demo
  :configs $ %{} :Config
    :debug true
    :port 8080
    :entries $ %{} :Entry
      :main $ %{} :Fn
        :name |main
        :args $ [] |x |y
        (:body |x)
      :reload $ %{} :Fn
        :name |reload
        :args $ []
        :body |reload-body
    :modules $ %{} :Module
      :core $ %{} :Dep (:version |1.0 :extra) (:path |/lib)
      :lib $ %{} :Dep
        :version |2.0
        :path |/lib2
    :hooks $ %{} :Hooks
      :start |start-hook
      :stop |stop-hook
  :version |1.0
  :authors $ [] |Alice |Bob
"#;
  match parse(deep_nested) {
    Ok(val) => println!("  Unexpected success: {val}"),
    Err(e) => println!("{e}\n"),
  }

  // ----------------------------------------------------------------
  // Case 3: Simple pair error — message body uses format_one_liner
  // ----------------------------------------------------------------
  println!("── Case 3: malformed pair inside a simple record ──\n");
  let simple_pair = r#"
%{} :Book
  :title |Calcit
  :pages (300 extra)
  :author |Alice
"#;
  match parse(simple_pair) {
    Ok(val) => println!("  Unexpected success: {val}"),
    Err(e) => println!("{e}\n"),
  }

  // ----------------------------------------------------------------
  // Case 4: List as record key — another {:?}→one-liner fix
  // ----------------------------------------------------------------
  println!("── Case 4: list used as record key ──\n");
  let list_as_key = r#"
%{} :Weird
  ([] :oops) |bad
"#;
  match parse(list_as_key) {
    Ok(val) => println!("  Unexpected success: {val}"),
    Err(e) => println!("{e}\n"),
  }
}
