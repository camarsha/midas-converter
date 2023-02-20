use midasio::read::file::FileView;
use std::fs;
use write_data::WriteData;
mod bitmasks;
mod mdpp16_scp;
mod sort;
mod write_data;
use clap::Parser;
use std::env;

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

    //    let output_file = &output_file.replace(".mid", ".csv");

    // see midasio package documentation for details
    let contents = fs::read(args.input_file).unwrap();
    // lets try to decompress the file
    let file_view = FileView::try_from(&contents[..]).unwrap();
    let sorter = sort::DataSort::new(args.output_file.to_string(), args.chunk_size);
    let mut data = sorter.sort_loop(&file_view);
    data.write_data();
}
