use crate::utils::Name;

pub trait Component {
    fn name(&self) -> Name;
}

impl Component for () {
    fn name(&self) -> Name {
        Name::new("()")
    }
}

impl Component for Name {
    fn name(&self) -> Name {
        self.clone()
    }
}
