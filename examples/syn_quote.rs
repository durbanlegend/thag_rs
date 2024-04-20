//! [dependencies]
//! quote = "1.0.36"
//! syn = { version = "2.0.60", features = ["full"] }

use quote::quote;
use std::io::Write;
use std::process::Command;
use syn::Expr;

fn main() {
    loop {
        println!("Enter an expression (e.g., 2 + 3): ");
        let mut input = String::new();
        std::io::stdin()
            .read_line(&mut input)
            .expect("Failed to read input");

        // Parse the expression string into a syntax tree
        let expr: Result<Expr, syn::Error> = syn::parse_str::<Expr>(&input.trim());
        match expr {
            Ok(expr) => {
                // Generate Rust code for the expression
                let rust_code = quote!(let result = #expr; println!("result={}", result););

                eprintln!("rust_code={rust_code}");

                // Write the generated code to a temporary file
                let mut file = std::fs::File::create("temp.rs").expect("Failed to create file");
                file.write_all(rust_code.to_string().as_bytes())
                    .expect("Failed to write to file");

                // // Compile the temporary file into a dynamic library
                // let output = Command::new("cargo")
                //     .arg("build")
                //     .arg("--lib")
                //     .output()
                //     .expect("Failed to compile");

                // if output.status.success() {
                //     // Load the library and access the compiled function
                //     let lib_result = unsafe {
                //         let lib = dlopen("temp.so", libc::RTLD_LAZY);
                //         if lib.is_null() {
                //             Err(format!("Failed to load library: {}", libc::dlerror()))
                //         } else {
                //             let get_result = dlsym(lib, "result\0".as_ptr() as *const _);
                //             if get_result.is_null() {
                //                 Err(format!("Failed to find symbol 'result'"))
                //             } else {
                //                 let result_fn = *(get_result as *mut usize);
                //                 Ok((*result_fn)())
                //             }
                //         }
                //     };

                //     // Print the result
                //     match lib_result {
                //         Ok(result) => println!("Result: {}", result),
                //         Err(err) => println!("Error: {}", err),
                //     }
                // } else {
                //     println!("Error: Compilation failed");
                // }

                // // Clean up temporary file (optional)
                // std::fs::remove_file("temp.rs").expect("Failed to remove temporary file");
            }
            Err(err) => println!("Error parsing expression: {}", err),
        }
    }
}
