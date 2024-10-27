use gio::prelude::FromVariant;
use gio::prelude::ToVariant;
use glib_proc_macro_test::interface;
use glib_proc_macro_test::method;

trait VariantCallable {
    fn call_method(&mut self, method: &str, arg: glib::Variant) -> glib::Variant;
}

struct Greeter {
}

#[interface]
impl Greeter {
    #[method]
    pub fn hello(&mut self, name: String) -> String {
        format!("hello, {name}!")
    }

    pub fn hello_variant(&mut self, args: glib::Variant) -> glib::Variant {
        let args: (String,) = FromVariant::from_variant(&args).unwrap();
        let res = self.hello(args.0);
        let res = ToVariant::to_variant(&res);
        res
    }
}

impl VariantCallable for Greeter {
    fn call_method(&mut self, method: &str, args: glib::Variant) -> glib::Variant {
        match method {
            "hello" => self.hello_variant(args),
            _ => panic!(),
        }
    }
}

fn main() {
    let mut greeter = Greeter{};
    let args = ToVariant::to_variant(&("fuad",));
    let res = greeter.call_method("hello", args);
    println!("{res}");
}