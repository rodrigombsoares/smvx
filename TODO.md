# TODO
## List
- [X] Implement definition
- [X] Implement indexing
- [X] Implement appending
- [X] Correct identation and line breaks
- [X] Logical reduction
    - `reduce!(OPERATOR, loop_var := START..END, expression)` macro
    - `reduce!(|, i := 1..3, d[i].pos = S3)` -> `d1.pos = S3 | d2.pos = S3 | d3.pos = S3`
    - `G (!s1.overflow & !s2.overflow & !s3.overflow);`
- [X] List unpacking
    - `MODULE packet(...servers)` and be able to use this list later

## Dynamic
- [X] Implement dynamic constructors (ie: `server_node(servers[0])`)
- [X] Expand macro `pos : {GS, ES, expand!(S{i}, i := 1..3)};`
    - Dynamic enums (eg: `pos : {GS, S1, S2, S3, D1, D2, D3};`)
- [X] Access a value from an enum dynamically (identifier interpolation)
    - `d[i].pos = GS : D{i};`

## For loops
- [X] Implement definition
- [X] Fix list indexing to accept variables

## Imports
- [X] Import from multiple files
    - project folder with a main file. Imports will be walked from main.

## Global
- [X] Global variables
    - Introduce global variables that can be re-written. eg:
    ```
    GLOBAL
        servers_amt = 7
    ...
    for i := 1..servers_amt do
    ```
    - for loops should support globals as start/end index
- [ ] Identifier interpolation for globals (currently only inside arg_expr)

## IR
- [X] Two passes in the lowerer
    - First pass picks up `Symbol Registration`, second pass `Expansion & Lowering`
    - This way a list defined under the `DEFINE` block can have elements picked up by `VAR` above

## CLI
- [X] Create CLI for transpiler

## General
- [X] Refactor ir.rs context
- [X] Refactor ir.rs methods
- [X] Rust proper error handling

## Bugs
- [X] `nodes := [node(0, triggers[0])];` not recognized by the grammar
- [X] Variable outside list assignment is not considered (maybe use a {i} here)
```
  for i := 1..2 do
    nodes.append(node(i, triggers[i]));
  od;
```
- [ ] If module doesn't end with \n then generated code should add this breaking
- [ ] Fix line break after global removal
- [X] Loop with reduce is not working (maybe nested loops are not working)
- [X] indentation errors on loops inside case
```
        token := case
            for i := 0..3 do
                request_charge_ds[i] : D{i};
            od;
            TRUE : NONE;
        esac;
```