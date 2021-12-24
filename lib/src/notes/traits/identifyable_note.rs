use std::hash::Hasher;

pub trait IdentifiableNote {
    fn folder(&self) -> String;
    fn uuid(&self) -> String;
}

pub trait Subject {
    fn first_subject(&self) -> String;
}

impl std::hash::Hash for Box<dyn IdentifiableNote> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.uuid().hash(state)
    }
}

impl std::cmp::PartialEq for Box<dyn IdentifiableNote>  {
    fn eq(&self, other: &Self) -> bool {
        self.uuid() == other.uuid()
    }

    fn ne(&self, other: &Self) -> bool {
        self.uuid() != other.uuid()
    }
}

impl std::cmp::Eq for Box<&dyn IdentifiableNote> {}

impl std::cmp::PartialEq for Box<&dyn IdentifiableNote>  {
    fn eq(&self, other: &Self) -> bool {
        self.uuid() == other.uuid()
    }

    fn ne(&self, other: &Self) -> bool {
        self.uuid() != other.uuid()
    }
}

impl std::hash::Hash for Box<&dyn IdentifiableNote> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.uuid().hash(state);
    }
}

