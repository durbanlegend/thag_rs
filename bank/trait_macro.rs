trait AlphabetLetter {
    fn some_function(&self);
}

macro_rules! make_alphabet {
    ($($x:ident),*) => {
        enum Letter {
            $(
                $x($x),
            )*
        }

        impl AlphabetLetter for Letter {
            fn some_function(&self) {
                match self {
                    $(
                        Letter::$x(letter) => println!("letter={letter}"),
                    )*
                }
            }
        }
    };
}

make_alphabet!("A", "B", "C");

let letter = Letter::A;
letter.some_function();
