mod demo {
    include!(concat!(env!("OUT_DIR"), "/demo/mod.rs"));
}
use demo::{Gender, User};
fn main() {
    let first_name = "f".to_string();
    let last_name = "l".to_string();
    let gender = Gender::Male;
    let age = 10;
    let active = true;
    let info = None;

    // let user = User::new(first_name, last_name, age, gender, active, info)
    let user = User {
        first_name,
        last_name,
        gender,
        age,
        active,
        info,
    };
    println!("user: {:?}", user);
}
