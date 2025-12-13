use cxx::let_cxx_string;

fn main() {
    let_cxx_string!(
        path = r"C:\Program Files (x86)\Steam\steamapps\common\Portal 2\portal2\pak01_dir.vpk"
    );

    let packfile = sourcepp::bridge::vpkpp::PackFile::open(&path);

    let_cxx_string!(nodraw = "materials/tools/toolsnodraw.vmt");

    let data = packfile.read_entry(&nodraw).unwrap();

    println!("{}", std::str::from_utf8(&data).unwrap());
}
