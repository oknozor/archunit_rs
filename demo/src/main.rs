use archunit_rs::Module;

mod module_one;
mod module_two;

fn main() {
    let module = Module::load_crate_root();
    println!("{module:#?}")
}
