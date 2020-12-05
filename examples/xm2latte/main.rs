mod xm;
mod intermediate;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: {} file.xm", args[0]);
        return;
    }

    let filename = &args[1];
    let xm = xm::File::load(filename).expect("Failed to load given file");
    println!("\
        // Automatically generated LatteFM source module.\n\
        //\n\
        // Original filename: {}",
        filename
    );
    xm.print_preamble();

    println!("");

    let module: intermediate::Module = From::from(xm);
    module.print_as_source();
}
