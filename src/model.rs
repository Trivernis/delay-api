#[derive(Clone, Debug)]
pub struct Station(pub String);

impl<S: AsRef<str>> From<S> for Station {
    fn from(value: S) -> Self {
        Self(value.as_ref().to_owned())
    }
}

#[derive(Clone, Debug)]
pub struct Line(pub String);

impl<S: AsRef<str>> From<S> for Line {
    fn from(value: S) -> Self {
        Self(value.as_ref().to_owned())
    }
}
