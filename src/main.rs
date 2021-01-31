mod args;

fn main() {
    let args = args::get_args();
    println!("{:#?}", args);
}
