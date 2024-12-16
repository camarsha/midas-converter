use midasio::read::file::FileView;
use smartstring::SmartString;
use std::fs;
mod bitmasks;
mod diagnostics;
mod mdpp_bank;
mod module_config;
mod sis3820;
mod sort;
mod v1730_bank;
mod v785_bank;
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
    #[arg(long, short)]
    output_file: Option<String>,
    config_file: String,
    #[arg(long, default_value_t = 10000000)]
    chunk_size: usize,
    #[arg(long, short, default_value_t = false)]
    diagnostic: bool,
    #[arg(long, short, default_value_t = false)]
    csv: bool,
    #[arg(long, short, default_value_t = false)]
    parquet: bool,
    #[arg(long, short, default_value_t = false)]
    feather: bool,
}

fn standard_convert(args: &mut Args, output_file: &mut str, pb: ProgressBar) {
    // create all of the output formats that the user wants

    // output parquet path buffer for the parquet_sink method.
    let output_file_parquet = Path::new(&format!(
        "{}{}",
        output_file.split('.').next().unwrap(),
        ".parquet"
    ))
    .to_path_buf();

    let output_file_feather = Path::new(&format!(
        "{}{}",
        output_file.split('.').next().unwrap(),
        ".feather"
    ))
    .to_path_buf();

    // This should stream the csv off the disk and periodically dump to the parquet file.
    let p_w_args = ParquetWriteOptions {
        compression: ParquetCompression::Lz4Raw,
        statistics: false,
        row_group_size: Some(60 * 1024 * 1024),
        data_pagesize_limit: None,
        maintain_order: true,
    };

    let ipc_args = IpcWriterOptions {
        compression: None,
        maintain_order: true,
    };

    let mut sch = Schema::new();

    let columns = [
        "module",
        "channel",
        "adc",
        "long",
        "short",
        "tdc",
        "trigger_dt",
        "pileup",
        "evt_ts",
    ];

    let col_types = [
        DataType::Int32,   // module id
        DataType::Int32,   // channel id
        DataType::Int32,   // adc
        DataType::Int32,   // long integral
        DataType::Int32,   // short integral
        DataType::Int32,   // tdc
        DataType::Int64,   // trigger dt
        DataType::Boolean, // Pileup flag
        DataType::Int64,   // Event or timestamp
    ];

    for (&col, dt) in columns.iter().zip(col_types.iter()) {
        let mut s = SmartString::new();
        s.push_str(col);
        sch.with_column(s, dt.to_owned());
    }

    if args.parquet {
        // Parquet dump
        LazyCsvReader::new(&output_file)
            .has_header(true)
            .with_dtype_overwrite(Some(&sch))
            .finish()
            .unwrap()
            .with_type_coercion(true)
            .sink_parquet(output_file_parquet, p_w_args)
            .expect("Error writing parquet file.");

        pb.println("Parquet conversion done!");
    }
    if args.feather {
        // feather dump
        LazyCsvReader::new(&output_file)
            .has_header(true)
            .with_dtype_overwrite(Some(&sch))
            .finish()
            .unwrap()
            .with_type_coercion(true)
            .sink_ipc(output_file_feather, ipc_args)
            .expect("Error writing feather file.");
        pb.finish_with_message("Feather conversion done!");
    }
    // delete the csv if you want to cover your tracks
    if !args.csv {
        Command::new("rm")
            .arg(&output_file)
            .status()
            .expect("Failed to delete csv file.");
    }
}

fn v1730_convert(args: &mut Args, output_file: &mut str, pb: ProgressBar) {
    // create all of the output formats that the user wants

    // output parquet path buffer for the parquet_sink method.
    let output_file_parquet = Path::new(&format!(
        "{}{}",
        output_file.split('.').next().unwrap(),
        ".parquet"
    ))
    .to_path_buf();

    let output_file_feather = Path::new(&format!(
        "{}{}",
        output_file.split('.').next().unwrap(),
        ".feather"
    ))
    .to_path_buf();

    // This should stream the csv off the disk and periodically dump to the parquet file.
    let p_w_args = ParquetWriteOptions {
        compression: ParquetCompression::Lz4Raw,
        statistics: false,
        row_group_size: Some(60 * 1024 * 1024),
        data_pagesize_limit: None,
        maintain_order: true,
    };

    let ipc_args = IpcWriterOptions {
        compression: None,
        maintain_order: true,
    };

    let mut sch = Schema::new();

    let columns = ["channel", "long", "coarse_time", "time"];

    let col_types = [
        DataType::Int32,   // channel id
        DataType::Int32,   // long integral
        DataType::Int64,   // coarse time
        DataType::Float64, // time
    ];

    for (&col, dt) in columns.iter().zip(col_types.iter()) {
        let mut s = SmartString::new();
        s.push_str(col);
        sch.with_column(s, dt.to_owned());
    }

    if args.parquet {
        // Parquet dump
        LazyCsvReader::new(&output_file)
            .has_header(true)
            .with_dtype_overwrite(Some(&sch))
            .finish()
            .unwrap()
            .with_type_coercion(true)
            .sink_parquet(output_file_parquet, p_w_args)
            .expect("Error writing parquet file.");

        pb.println("Parquet conversion done!");
    }
    if args.feather {
        // feather dump
        LazyCsvReader::new(&output_file)
            .has_header(true)
            .with_dtype_overwrite(Some(&sch))
            .finish()
            .unwrap()
            .with_type_coercion(true)
            .sink_ipc(output_file_feather, ipc_args)
            .expect("Error writing feather file.");
        pb.finish_with_message("Feather conversion done!");
    }
    // delete the csv if you want to cover your tracks
    if !args.csv {
        Command::new("rm")
            .arg(&output_file)
            .status()
            .expect("Failed to delete csv file.");
    }
}

fn main() {
    // parse the command line args
    let mut args = Args::parse();
    // if an output filename was not passed, then generate it from the input
    if args.output_file.is_none() {
        // just to remind myself later, split the string, get the first Option<element> before . with next
        let temp_filename = format!("{}{}", args.input_file.split('.').next().unwrap(), ".csv");
        args.output_file = Some(temp_filename);
    }

    // Now move the value out of the struct so that we don't have to jump through hoops.
    let mut output_file = args.output_file.clone().unwrap();
    let scaler_output_file = format!("{}_scaler.csv", output_file.split('.').next().unwrap());
    // if it is user supplied, make sure it has a .csv
    if !output_file.contains(".csv") {
        output_file = format!("{}{}", output_file, ".csv");
    }

    // decompress the file in the most janky way possible
    let filename = if args.input_file.contains("lz4") {
        Command::new("lz4")
            .arg("-d")
            .arg("-f")
            .arg(args.input_file.clone())
            .status()
            .unwrap_or_else(|_| panic!("The command lz4 -d -f {} failed!", args.input_file.clone()));
        println!("{}", args.input_file.replace(".lz4", ""));
        args.input_file.replace(".lz4", "")
    } else {
        args.input_file.clone()
    };

    // see midasio package documentation for details
    let contents = fs::read(&filename).expect("Failed to read midas file.");
    let file_view = FileView::try_from(&contents[..]).expect("Failed to create FileView of midas file.");
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

    // get the configuration, which will choose the type of sorter to use
    let config: module_config::Config = module_config::create_config(&args.config_file);

    // initialize the sorter
    let mut is_adc = false;
    let mut is_v1730 = false;
    for m in config.modules.iter() {
        if m.mod_type == "adc" {
            is_adc = true;
            break;
        } else if m.mod_type == "v1730" {
            is_v1730 = true;
            break;
        }
    }

    // Dont be stupid
    if is_adc & is_v1730 {
        panic!("Configuration has both adc and v1730 modules!!")
    };

    if is_adc {
        sort::v785Sort::new(
            output_file.clone(),
            scaler_output_file,
            args.chunk_size,
            config,
        )
        .sort_loop(&file_view);
    } else if is_v1730 {
        sort::v1730Sort::new(
            output_file.clone(),
            scaler_output_file,
            args.chunk_size,
            config,
        )
        .sort_loop(&file_view);
    } else {
        sort::MDPPSort::new(
            output_file.clone(),
            scaler_output_file,
            args.chunk_size,
            config,
        )
        .sort_loop(&file_view);
    }

    // remove file if we created it
    if args.input_file.contains("lz4") {
        Command::new("rm")
            .arg(Path::new(&filename))
            .status()
            .expect("Failed to delete file.");
    }

    // spinner while we convert
    let pb = ProgressBar::new_spinner();
    pb.enable_steady_tick(Duration::from_millis(200));
    pb.tick();
    pb.set_message("Converting to final formats...");

    if is_v1730 {
        v1730_convert(&mut args, &mut output_file, pb)
    } else {
        standard_convert(&mut args, &mut output_file, pb)
    }
}
