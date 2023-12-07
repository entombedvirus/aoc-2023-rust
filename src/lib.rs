use std::path::Path;

use anyhow::Result;

pub fn runner<A: std::fmt::Display, B: std::fmt::Display>(
    part_one: fn(&str) -> Result<A>,
    part_two: fn(&str) -> Result<B>,
) -> Result<()> {
    let mut args = std::env::args();
    let binary_path = args.next().expect("binary name to be present");
    let binary_name = Path::new(&binary_path)
        .file_name()
        .and_then(|p| p.to_str())
        .expect("file_name to_str failed");
    let cmd = args
        .next()
        .expect("usage: cmd [1|2] [input_file_path]. cmd is missing");
    let input_file_path = args
        .next()
        .unwrap_or_else(|| format!("inputs/{}.txt", binary_name));
    let input = std::fs::read_to_string(&input_file_path)
        .expect(format!("unable to read input file: {input_file_path}").as_str());
    match cmd.as_str() {
        "1" => {
            println!("{}", part_one(&input)?);
        }
        "2" => {
            println!("{}", part_two(&input)?);
        }
        u => {
            anyhow::bail!("unknown cmd: {u}");
        }
    };
    Ok(())
}

pub fn wait() {
    let _ = std::io::stdin().read_line(&mut String::new()).unwrap();
}
