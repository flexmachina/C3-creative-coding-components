use webxr_app::run;

fn main() {
    println!("Hello, world!");
    pollster::block_on(run());
}
