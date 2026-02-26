use critical_section as _;

fn main() {
    critical_section::with(|_| {
        println!("Hello, world!");
    });
}
