# smvx

## Testing
Test printing pest tree

```shell
cargo test -- --nocapture
```

## TODO
### List
- [X] Implement definition
- [X] Implement indexing
- [X] Implement appending
- [X] Correct identation and line breaks

### Dynamic
- [X] Implement dynamic constructors (ie: `server_node(servers[0])`)

### For loops
- [X] Implement definition
- [X] Fix list indexing to accept variables

### Imports
- [] Import from multiple files
    - project folder with a main file. Imports will be walked from main.

### General
- [X] Refactor ir.rs context
- [X] Refactor ir.rs methods
- [X] Rust proper error handling