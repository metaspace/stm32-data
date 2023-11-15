use chiptool::ir;
use chiptool::generate;
use chiptool::transform;
use regex::Regex;
use std::io::Write;
use std::str::FromStr;
use std::string::ToString;
use proc_macro2::TokenStream;
use std::fs::File;

fn main() {
    let mut ir: ir::IR = serde_yaml::from_reader(File::open("../kernel/nvme.yaml").unwrap()).unwrap();

    let options = 
        generate::Options {
            common_module: generate::CommonModule::External(TokenStream::from_str("regs_rt").unwrap()),
        };

    transform::expand_extends::ExpandExtends {}.run(&mut ir).unwrap();

    transform::map_names(&mut ir, |k, s| match k {
        transform::NameKind::Block => *s = s.to_string(),
        transform::NameKind::Fieldset => *s = format!("regs::{}", s),
        transform::NameKind::Enum => *s = format!("vals::{}", s),
        _ => {}
    });

    transform::sort::Sort {}.run(&mut ir).unwrap();
    transform::Sanitize {}.run(&mut ir).unwrap();

    let items = generate::render(&ir, &options).unwrap();
    let mut file = File::create("generated.rs").unwrap();
    let data = items.to_string().replace("] ", "]\n");

    // Remove inner attributes like #![no_std]
    let data = Regex::new("# *! *\\[.*\\]").unwrap().replace_all(&data, "");

    file.write("pub mod regs_rt;\n".as_bytes()).unwrap();
    file.write_all(data.as_bytes()).unwrap();


    std::fs::write(
       "common.rs",
        chiptool::generate::COMMON_MODULE,
    ).unwrap()

}
