use std::{
    env,
    io::{self},
};

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let mut input_path = String::from("transactions.csv");
    let mut output_path = String::from("accounts.csv");
    if let Some(input_file_path) = args.get(1) {
        input_path = input_file_path.clone();
    }

    if let Some(output_file_path) = args.get(2) {
        output_path = output_file_path.clone();
    }

    let result = service::service::read_csv(input_path).expect("csv error");
    service::service::write_csv(output_path, &result).expect("csv error");

    Ok(())
}
