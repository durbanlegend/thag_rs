let strings = vec!["7", "42", "one"];
let numbers: Result<Vec<i32>, _> = strings
    .into_iter()
    .map(|s| s.parse::<i32>())
.inspect(|x| eprintln!("{x:?}"))
.collect::<Result<Vec<_>, _>>(); // This collects into a Result<Vec<i32>, ParseIntError>
numbers
