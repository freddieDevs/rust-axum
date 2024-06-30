//! Simplistic Model Layer
//! (with mock-store layer)


use crate::{ctx::Ctx, Error, Result};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

// region:  --- Ticket Types
#[derive(Clone, Debug, Serialize)]
pub struct Ticket {
  pub id: u64,
  pub cid: u64, // creator user_id
  pub title: String,
}
#[derive(Deserialize)]
pub struct TicketForCreate {
  pub title: String
  // we only use deserialize to only deserialize the data
}
// endregion: --- Ticket Types

// region:  --- Model Controller
#[derive(Clone)]
pub struct  ModelController {
  ticket_store: Arc<Mutex<Vec<Option<Ticket>>>>,
}

// Constructor
impl ModelController {
    pub async fn new() ->Result<Self> {
        Ok(Self {
          ticket_store: Arc::default(),
        })
    }
}

// CRUD IMPL
impl ModelController {
    pub async fn  create_ticket(
      &self,
      ctx: Ctx,
      ticket_fc: TicketForCreate) -> Result<Ticket> {
        let mut store = self.ticket_store.lock().unwrap();

        let id = store.len() as u64;
        let ticket = Ticket {
          id,
          cid: ctx.user_id(),
          title: ticket_fc.title,
        };
        // push the ticket into the store as an option
        store.push(Some(ticket.clone()));
        // return the original as an Ok
        Ok(ticket)
    }
    
    pub async fn list_tickets(&self, _ctx: Ctx) -> Result<Vec<Ticket>> {
        let store = self.ticket_store.lock().unwrap();

        let tickets = store.iter().filter_map(|t| t.clone()).collect();

        Ok(tickets)
    }
  
    pub async fn delete_ticket(&self, id: u64, _ctx: Ctx) -> Result<Ticket> {
        // get the store mutably
        let mut store = self.ticket_store.lock().unwrap();

        // get the deleted ticket
        let ticket = store.get_mut(id as usize).and_then(|t| t.take());

        ticket.ok_or(Error::TicketDeleteFailIdNotFound { id })
    }
  }
// endregion  --- Model Controller