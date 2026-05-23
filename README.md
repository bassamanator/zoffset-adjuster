```rust
    impl GCode {
        const LAYER_CHANGE: &'static str = ";LAYER_CHANGE";
        const CURRENT_PRINT_HEIGHT: &'static str = ";Z:";
        const CURRENT_LAYER_HEIGHT: &'static str = ";HEIGHT:";
    }
    // NOTE example sequence found in OS 2.3.2
    // first layer height is 0.22
    // layer height is 0.12
    // NOTE first layer
    // ;LAYER_CHANGE
    // ;Z:0.22
    // ;HEIGHT:0.22
    // NOTE third layer
    // ;LAYER_CHANGE
    // ;Z:0.46
    // ;HEIGHT:0.12
```
