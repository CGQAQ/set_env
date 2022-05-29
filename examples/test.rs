use set_env;

fn main() {
    // set_env::check_or_set("d", "world6777").expect("not working");
    set_env::set("XXX", "man").unwrap();
    set_env::set("XXXd", "man").unwrap();
    set_env::append("PATH", "hello").unwrap();
    set_env::prepend("PATH", "okkkk").unwrap();
}
