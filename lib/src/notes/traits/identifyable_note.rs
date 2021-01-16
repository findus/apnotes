use std::hash::Hasher;

pub trait IdentifyableNote {
    fn folder(&self) -> String;
    fn uuid(&self) -> String;
}

impl std::hash::Hash for Box<dyn IdentifyableNote> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.uuid().hash(state)
    }
}

impl std::cmp::PartialEq for Box<dyn IdentifyableNote>  {
    fn eq(&self, other: &Self) -> bool {
        self.uuid() == other.uuid()
    }

    fn ne(&self, other: &Self) -> bool {
        self.uuid() != other.uuid()
    }
}

impl std::cmp::Eq for Box<&dyn IdentifyableNote> {}

impl std::cmp::PartialEq for Box<&dyn IdentifyableNote>  {
    fn eq(&self, other: &Self) -> bool {
        self.uuid() == other.uuid()
    }

    fn ne(&self, other: &Self) -> bool {
        self.uuid() != other.uuid()
    }
}

impl std::hash::Hash for Box<&dyn IdentifyableNote> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.uuid().hash(state);
    }
}

