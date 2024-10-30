# riverbed_block_def
Turns an asset file containing custom block definitions into Rust code that define Blocks + their families + some useful functions.

Other stuff that is not expected to change often is defined by hand in a separate Blocks impl.

## Syntax
### Define a set 
```rust
set Wood {
    Acacia,
    Birch,
    Oak,
    Sequoia,
    Spruce
}
```

### Define a block
This will automatically define a "Planks" for every Wood, as well as register "Wood" as a BlockFamily for those blocks.
```rust
block {Wood}Planks
```

Some flags are also available such as:
```rust
block GoldOre renewable(30)
```
which will define GoldOre as a block that can be harvested and renews itself in 30 minutes.