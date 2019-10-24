use super::Element;

pub trait Component {
    fn render(&self) -> Element;
}