//! The whole of a `rui` program, in a screenful.
//!
//! `cargo run -p rui --example counter` opens it.
//!
//! State in, description out, and a loop that runs the two: that is the whole
//! of the shape, and it is all that is left here. The description itself — the
//! `Counter` and the `El` tree that is a pure function of it — is
//! [`rui::demo::counter_view`], read it there.
//!
//! It lives in the library rather than in this file so that this example and
//! the browser build in `src/wasm.rs` are not two similar programs but one
//! program with two drivers. `examples/parity.rs` then draws that same
//! description to a PNG with no window at all, and `examples/parity.html`
//! checks — byte for byte — that a browser puts the identical picture on a
//! `<canvas>`. A copy of the view pasted into each backend would have made that
//! comparison meaningless the first time one copy was edited.

use rui::demo::{counter_view, Counter};

fn main() -> Result<(), rui::Error> {
    rui::run("Counter", Counter { count: 0 }, counter_view)
}
