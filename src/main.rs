#[macro_use]
extern crate lazy_static;

mod config;

fn main() {
    println!("token {}", config::CONF.token());
}
