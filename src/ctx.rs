#[derive(Clone, Debug)]
pub struct Ctx {
  user_id: u64,
}

// Constructor
impl Ctx {
    // takes the user_id & returns the context of it
    pub fn new(user_id: u64) -> Self{
      Self { user_id }
    }
}

// implementation of property accessors
// this one returns the id
impl Ctx {
  pub fn user_id(&self) -> u64 {
    self.user_id
    // by doing this we ensure nobody can 
    // change the id without being inside this module
  }
}