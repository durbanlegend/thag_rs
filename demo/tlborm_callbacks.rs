/// `Callbacks` example from `The Little Book of Rust Macros`
//# Purpose: Demo macro callbacks.
//# Categories: learning, technique
macro_rules! call_with_larch {
    ($callback:ident) => {
        $callback!(larch)
    };
}

#[allow(unused_macros)]
macro_rules! expand_to_larch {
    () => {
        larch
    };
}

macro_rules! recognise_tree {
    (larch) => {
        println!("#1, the Larch.")
    };
    (redwood) => {
        println!("#2, the Mighty Redwood.")
    };
    (fir) => {
        println!("#3, the Fir.")
    };
    (chestnut) => {
        println!("#4, the Horse Chestnut.")
    };
    (pine) => {
        println!("#5, the Scots Pine.")
    };
    ($($other:tt)*) => {
        println!("I don't know; some kind of birch maybe?")
    };
}

fn main() {
    recognise_tree!(expand_to_larch!());
    call_with_larch!(recognise_tree);
}
