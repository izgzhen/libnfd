extern crate libnfd;
use libnfd::*;

fn main() {
    match open_dialog_multiple(Some("png,jpg;pdf"), None) {
        Ok(paths) => println!("Paths: {:?}", paths),
        Err(e)    => println!("Errors: {:?}", e)
    }

    let last_err = get_error();
    println!("last error: {:?}", last_err);
}
