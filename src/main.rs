use midasio::read::file::FileView;
use std::fs;
use write_data::WriteData;
mod bitmasks;
mod mdpp16_scp;
mod sort;
mod write_data;

fn main() {
    // and there is a package...
    let contents = fs::read("/home/caleb/midas-tests/run00105.mid").unwrap();
    // lets try to decompress the file
    let file_view = FileView::try_from(&contents[..]).unwrap();
    let sorter = sort::DataSort::new("test.csv".to_string(), 100);
    let mut data = sorter.sort_loop(&file_view);
    data.write_data("test.csv");
}
