use midasio::read::file::FileView;
use std::fs;
use write_data::WriteData;
mod bitmasks;
mod mdpp16_scp;
mod sort;
mod write_data;
use std::env;

fn main() {
    // parse the command line args
    let args: Vec<String> = env::args().collect();

    let input_file: &str = &args[1];
    let output_file: &str = if args.len() >= 3 {
        &args[2]
    } else {
        panic!("Need output file name!")
    };

    //    let output_file = &output_file.replace(".mid", ".csv");

    // see midasio package documentation for details
    let contents = fs::read(input_file).unwrap();
    // lets try to decompress the file
    let file_view = FileView::try_from(&contents[..]).unwrap();
    let sorter = sort::DataSort::new(output_file.to_string(), 100);
    let mut data = sorter.sort_loop(&file_view);
    data.write_data(output_file);
}
