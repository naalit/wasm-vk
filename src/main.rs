use std::borrow::Borrow;
use wasm_vk::*;

struct Args {
    short: Vec<char>,
    long: Vec<String>,
    /// Stored backwards
    rest: Vec<String>,
}
impl Args {
    fn flag(&self, short: impl Borrow<char>, long: impl Into<String>) -> bool {
        self.short.contains(short.borrow()) || self.long.contains(&long.into())
    }
    fn next(&mut self) -> Option<String> {
        self.rest.pop()
    }
}

fn args() -> Args {
    let mut args = std::env::args();
    let mut long = Vec::new();
    let mut short = Vec::new();
    let mut rest = Vec::new();
    // Skip the executable name
    args.next();
    for arg in args {
        if arg.starts_with("--") {
            long.push(arg[2..].to_owned());
        } else if arg.starts_with('-') {
            for i in arg[1..].chars() {
                short.push(i);
            }
        } else {
            rest.push(arg);
        }
    }
    rest.reverse();
    Args { short, long, rest }
}

fn main() {
    let mut args = args();
    let verbose = args.flag('v', "verbose");
    let in_file = args.next().expect("No input file!");
    let out_file = args.next().unwrap_or_else(|| String::from("out.spv"));

    if verbose {
        println!(
            "Deserializing WASM file {} to SPIR-V file {}",
            in_file, out_file
        );
    }

    // We get our WASM from the 'comp.wasm' file, which is compiled from 'comp.wat'
    // It multiplies every number by 12 and adds 3
    let w = wasm::deserialize_file(in_file)
        .expect("Deserialization error: are you sure this is valid WASM?");

    if verbose {
        println!("Deserialized WASM: {:?}", w);
    }

    // First, we generate SPIR-V
    let spv = spirv::to_spirv(w.clone());

    if verbose {
        use rspirv::binary::Disassemble;
        let mut l = rspirv::dr::Loader::new();
        rspirv::binary::parse_bytes(&spv, &mut l).unwrap();
        println!("Dissasembled SPIR-V:\n{}", l.module().disassemble());
    }

    // We write the SPIR-V to disk so we can disassemble it later if we want
    use std::io::Write;
    let mut f = std::fs::File::create(&out_file).unwrap();
    f.write_all(&spv).unwrap();

    if verbose {
        println!("Written generated spirv to '{}'", out_file);
    }
}
