// A helper struct that will call a function when it is dropped
#[must_use = "The value must be held in the scope"]
pub struct OnceDestructor<F: FnOnce()> {
    call_on_drop: Option<F>,
}

impl<F: FnOnce()> OnceDestructor<F> {
    pub fn new(call_on_drop: F) -> Self {
        Self {
            call_on_drop: Some(call_on_drop),
        }
    }
}

impl<F: FnOnce()> Drop for OnceDestructor<F> {
    fn drop(&mut self) {
        self.call_on_drop.take().expect("Must exist")();
    }
}
