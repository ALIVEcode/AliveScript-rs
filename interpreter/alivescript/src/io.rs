use crate::data::{Data, Response};

pub trait InterpretorIO {
    fn send(&mut self, data: Data);
    fn request(&mut self, data: Data) -> Option<Response>;
}
