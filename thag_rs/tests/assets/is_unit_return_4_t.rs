fn foo() -> bool {
    let truth_value = {
        for i in 0..30 {
            if i % 23 == 0 {
                return true;
            }
        }
        false
    };
    truth_value
}

foo()
