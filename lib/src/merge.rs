

pub fn merge_two(first: &str, second: &str) -> String {
    diff::lines(first, second).iter().map( { |diff|
        match diff {
            diff::Result::Left(l) => { format!("< {}", l) },
            diff::Result::Both(l, _) => format!("{}", l),
            diff::Result::Right(r)   => format!("> {}", r)
        }
    })
        .map(|string| format!("{}\n", string))
        .collect()
}