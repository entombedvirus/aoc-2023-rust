use anyhow::Result;

pub fn runner<A: std::fmt::Display, B: std::fmt::Display>(
    part_one: fn(&str) -> Result<A>,
    part_two: fn(&str) -> Result<B>,
) -> Result<()> {
    let mut args = std::env::args();
    let cmd = args
        .nth(1)
        .expect("usage: cmd [1|2] input_file_path. cmd is missing");
    let input_file_path = args
        .next()
        .expect("usage: cmd [1|2] input_file_path. input_file_path is missing");
    let input = std::fs::read_to_string(input_file_path).expect("unable to read input file");
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
