/// Use a trait to determine the type of an expression at compile time, provided all cases are known in advance.
///
/// This is a slightly embellished version of user `phicr`'s answer on `https://stackoverflow.com/questions/21747136/how-do-i-print-the-type-of-a-variable-in-rust`.
///
/// See also `demo/type_of_at_compile_time_2.rs` for an alternative implementation.
//# Purpose: Demo expression type determination for static dispatch.
use std::stringify;

trait TypeInfo {
    fn type_name() -> String;
    fn type_of(&self) -> String;
}

macro_rules! impl_type_info {
    ($($name:ident$(<$($T:ident),+>)*),*) => {
        $(impl_type_info_single!($name$(<$($T),*>)*);)*
    };
}

macro_rules! mut_if {
    ($name:ident = $value:expr, $($any:expr)+) => {
        let mut $name = $value;
    };
    ($name:ident = $value:expr,) => {
        let $name = $value;
    };
}

macro_rules! impl_type_info_single {
    ($name:ident$(<$($T:ident),+>)*) => {
        impl$(<$($T: TypeInfo),*>)* TypeInfo for $name$(<$($T),*>)* {
            fn type_name() -> String {
                mut_if!(res = String::from(stringify!($name)), $($($T)*)*);
                $(
                    res.push('<');
                    $(
                        res.push_str(&$T::type_name());
                        res.push(',');
                    )*
                    res.pop();
                    res.push('>');
                )*
                res
            }
            fn type_of(&self) -> String {
                $name$(::<$($T),*>)*::type_name()
            }
        }
    }
}

impl<'a, T: TypeInfo + ?Sized> TypeInfo for &'a T {
    fn type_name() -> String {
        let mut res = String::from("&");
        res.push_str(&T::type_name());
        res
    }
    fn type_of(&self) -> String {
        <&T>::type_name()
    }
}

impl<'a, T: TypeInfo + ?Sized> TypeInfo for &'a mut T {
    fn type_name() -> String {
        let mut res = String::from("&mut ");
        res.push_str(&T::type_name());
        res
    }
    fn type_of(&self) -> String {
        <&mut T>::type_name()
    }
}

macro_rules! type_of {
    ($x:expr) => {
        (&$x).type_of()
    };
}

// NB: Set up the types we want to check
impl_type_info!(i32, i64, f32, f64, str, String, Vec<T>, Result<T,S>);

fn main() {
    println!("Type of {} is {}", stringify!(1), type_of!(1));
    println!("Type of {} is {}", stringify!(&1), type_of!(&1));
    println!("Type of {} is {}", stringify!(&&1), type_of!(&&1));
    println!("Type of {} is {}", stringify!(&mut 1), type_of!(&mut 1));
    println!("Type of {} is {}", stringify!(&&mut 1), type_of!(&&mut 1));
    println!("Type of {} is {}", stringify!(&mut &1), type_of!(&mut &1));
    println!("Type of {} is {}", stringify!(1.0), type_of!(1.0));
    println!("Type of {} is {}", stringify!("abc"), type_of!("abc"));
    println!("Type of {} is {}", stringify!(&"abc"), type_of!(&"abc"));
    println!(
        "Type of {} is {}",
        stringify!(String::from("abc")),
        type_of!(String::from("abc"))
    );
    println!(
        "Type of {} is {}",
        stringify!(vec![1, 2, 3]),
        type_of!(vec![1, 2, 3])
    );
    println!(
        "Type of {} is {}",
        stringify!({ 2 + 3 }),
        type_of!({ 2 + 3 })
    );
}
