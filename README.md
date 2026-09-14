# SMVX

> [!NOTE]  
> This README was written by a human for humans (i.e. I've tried to make this human readable)

SMVX is an extension to the [NuSMV](https://nusmv.fbk.eu/) language. It introduces missing features such as `for` loops, lists and imports. This repo contains a Rust CLI that transpiles SMVX code into a regular NuSMV file. 

## Usage

**1. Install the CLI**
```
cargo install --git https://github.com/rodrigombsoares/smvx.git
```
> Use `--force` to upgrade an existing installation.

**2. Transpile**
Write your modular `.smvx` code in multiple files, use import statements and generate compiled NuSMV output by running this command agains `main.smvx`

```shell
smvx -s <path-to-smvx-file> -o <path-to-output-smv>
```

## Features
### Imports  
Enable modularity by splitting modules definitions across files and importing them into a main module.
```
-- main.smvx
import "counter/counter.smvx";

MODULE main
...
```
### List 

#### definition
define a list inside `VAR` by using the syntax `servers := [...];`

#### append
append an element to a list eg: `servers.append(server_node(FALSE));`

#### indexing
reference elements dynamically using `my_list[i]`   
(where `i` is a number or a loop variable)

### Dynamic constructors
Pass variables dynamically to module constructors (e.g. `server_node(servers[0])`)

### For loops
Unroll repetitive definitions. Use this syntax to define a for loop:
```
  for i := 1..2 do
    ...
  od;
```

## Examples
Examples can be found in the `tests/fixtures` directory. The generated `.svm` files represent the expected transpiled results.

## Architecture
When calling SMVX tramspiler 

**CLI → Transpiler → Parse (Pest) → AST → Lowering → IR → Codegen**

### Parsing -> AST
The SMVX transpiler uses [pest](https://pest.rs/) for parsing. The grammar covers the extension blocks while leaving NuSMV syntax as `raw nusvm` chunks.

> [!WARNING]  
> It might be the case that some standard features collide with our grammar definition, as we haven't exhaustively covered the entire NuSMV spec in testing. Please open an issue if you find a parsing bug.  
   
SMVX code is parsed into an AST structure containing all the extension blocks, plus raw nusmv code.

### Lowering -> IR
The AST is then lowered into a sort of an Intermediate Representation (IR). High level definitions such lists and loops are expanded into multiple variables. For example:

This `.smvx`
```
servers := [server_node(TRUE), server_node(FALSE), server_node(FALSE)];
```

Is transformed into multiple IR variable declarations
```rust
VarDecl { indent: String, name: String, value: String },
```

### Codegen
IR chunks are then formatted into standard NuSMV text. A var declarations from the example above, will be transformed into
```
  server0 : server_node(TRUE);
  server1 : server_node(FALSE);
  server2 : server_node(FALSE);
```

## Testing
Run the test suite to verify the parser and AST tree generation:

```shell
cargo test -- --nocapture
```
