pub mod postgres;

pub trait Escaper {
    fn escape_value(&self, value: String) -> String;
}
