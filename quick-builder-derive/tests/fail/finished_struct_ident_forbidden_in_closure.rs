use quick_builder_derive::QuickBuilder;

#[derive(QuickBuilder)]
struct Foo {
    #[invariant(|val|{
        {
           if *val>0. {
               // we hide the usage away in the AST to make sure the
               // parsing works correctly
               let f = __finished_instance;
           }
        }
        *val>0.})]
    x: f32,
}

fn main() {}
