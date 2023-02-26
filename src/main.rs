use midasio::read::file::FileView;
use std::fs;
use write_data::WriteData;
mod bitmasks;
mod mdpp16_scp;
mod sort;
mod write_data;
use clap::Parser;
use std::env;
use std::process::Command;

// I ripped this straight from the clap documentation
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    input_file: String,
    output_file: String,
    #[arg(long, default_value_t = 100000)]
    chunk_size: usize,
}

fn main() {
    // parse the command line args
    let args = Args::parse();

    // decompress the file in the most janky way possible
    let filename = if args.input_file.contains("lz4") {
        println!("Hello?");
        Command::new("lz4")
            .arg("-d")
            .arg("-f")
            .arg(args.input_file.clone())
            .status()
            .expect("Decompression failed");
        println!("{}", args.input_file.replace(".lz4", ""));
        args.input_file.replace(".lz4", "")
    } else {
        args.input_file.clone()
    };

    // see midasio package documentation for details
    let contents = fs::read(&filename).unwrap();
    let file_view = FileView::try_from(&contents[..]).unwrap();
    let sorter = sort::DataSort::new(args.output_file.to_string(), args.chunk_size);
    let mut data = sorter.sort_loop(&file_view);
    data.write_data();
    // remove file if we created it
    if args.input_file.contains("lz4") {
        Command::new("rm")
            .arg(filename)
            .status()
            .expect("Failed to delete file.");
    }
}
