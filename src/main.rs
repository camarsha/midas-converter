use midasio::read::file::FileView;
use std::fs;
mod bitmasks;
mod diagnostics;
mod mdpp_bank;
mod module_config;
mod sort;
mod write_data;
use clap::Parser;
use indicatif::ProgressBar;
use polars::prelude::*;
use std::path::Path;
use std::process::exit;
use std::process::Command;
use std::time::Duration;

// I ripped this straight from the clap documentation
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    input_file: String,
    output_file: String,
    config_file: String,
    #[arg(long, default_value_t = 10000000)]
    chunk_size: usize,
    #[arg(long, short, default_value_t = false)]
    diagnostic: bool,
    #[arg(long, short, default_value_t = true)]
    csv: bool,
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
    sorter.sort_loop(&file_view);
    // remove file if we created it
    if args.input_file.contains("lz4") {
        Command::new("rm")
            .arg(filename)
            .status()
            .expect("Failed to delete file.");
    }
    // output parquet path buffer for the parquet_sink method.
    let output_file_wo_csv = Path::new(&format!(
        "{}{}{}",
        "./",
        args.output_file.split('.').next().unwrap(),
        ".feather"
    ))
    .to_path_buf();
    //let mut p_file = File::create(output_file_wo_csv).expect("could not create file");

    // spinner while we convert
    let pb = ProgressBar::new_spinner();
    pb.enable_steady_tick(Duration::from_millis(200));
    pb.tick();
    pb.set_message("Converting to feather...");
    // This should stream the csv off the disk and periodically dump to the parquet file.
    // let p_w_args = ParquetWriteOptions {
    //     compression: ParquetCompression::Lz4Raw,
    //     statistics: false,
    //     row_group_size: Some(60 * 1024 * 1024),
    //     data_pagesize_limit: None,
    //     maintain_order: true,
    // };

    let ipc_args = IpcWriterOptions {
        compression: None,
        maintain_order: true,
    };

    // It works for ipc, weird...
    LazyCsvReader::new(&args.output_file)
        .finish()
        .unwrap()
        .sink_ipc(output_file_wo_csv, ipc_args)
        .expect("Error writing feather file.");

    pb.finish_with_message("Conversion done!");
    if !args.csv {
        Command::new("rm")
            .arg(&args.output_file)
            .status()
            .expect("Failed to delete csv file.");
    }
}
