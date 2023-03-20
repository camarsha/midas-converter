use midasio::read::file::FileView;
use std::fs;
use write_data::WriteData;
mod bitmasks;
mod diagnostics;
mod mdpp_bank;
mod module_config;
mod sort;
mod write_data;
use clap::Parser;
use std::collections::HashMap;
use std::process::exit;
use std::process::Command;

// I ripped this straight from the clap documentation
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    input_file: String,
    output_file: String,
    config_file: String,
    #[arg(long, default_value_t = 100000)]
    chunk_size: usize,
    #[arg(long, short, default_value_t = false)]
    diagnostic: bool,
}

fn main() {
    // parse the command line args
    let args = Args::parse();

    // decompress the file in the most janky way possible
    let filename = if args.input_file.contains("lz4") {
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
    // if we want diagnostics
    if args.diagnostic {
        let (one, two, three) = diagnostics::event_diagnostics(&file_view);
        println!("Banks: {}, Headers: {}, Event Ends: {}", one, two, three);
        // remove file if we created it
        if args.input_file.contains("lz4") {
            Command::new("rm")
                .arg(filename)
                .status()
                .expect("Failed to delete file.");
        }
        exit(0);
    }

    // initialize the sorter
    let sorter = sort::DataSort::new(
        args.output_file.to_string(),
        args.chunk_size,
        args.config_file.to_string(),
    );
    // sort the data
    let data = sorter.sort_loop(&file_view);
    // write out everything that remains. Let the user know something might be wrong
    // if there are still incomplete events.
    for (_key, mut value) in data {
        // start signals that a header has been read, but not an end
        // of event bank
        if !value.start {
            value.write_data();
        } else {
            println!("Malformed data is likely present. Number of event headers does not match number of end events.")
        }
    }
    // remove file if we created it
    if args.input_file.contains("lz4") {
        Command::new("rm")
            .arg(filename)
            .status()
            .expect("Failed to delete file.");
    }
}
