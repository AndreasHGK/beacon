use std::{collections::HashMap, sync::Arc};

use chrono::{DateTime, Duration, Utc};
use rand::random;
use serde::{Deserialize, Serialize};
use ssh_key::Fingerprint;
use tokio::{sync::Mutex, time::sleep};
use uuid::Uuid;

/// The state needed for SSH authentication.
pub struct SSHAuthState {
    tickets: Mutex<HashMap<Ticket, Fingerprint>>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct Ticket {
    /// A randomly generated nonce.
    nonce: u128,
    /// A unix timestamp in milliseconds.
    timestamp: i64,
    /// The 'owner' of the ticket.
    ///
    /// This is the user the session will be granted to on success.
    subject: Uuid,
}

impl SSHAuthState {
    /// Initialize a new [SSHAuthState] along with a cleanup task that cleans up old, unused
    /// tickets.
    pub fn init() -> Arc<Self> {
        let state = Arc::new(Self {
            tickets: Default::default(),
        });

        // Clean up old unused tickets periodically.
        let weak_state = Arc::downgrade(&state);
        tokio::spawn(async move {
            loop {
                sleep(std::time::Duration::from_secs(60)).await;
                // Check if the state is still used, stop the task otherwise. This is a relatively
                // simplistic way of ensuring this task is exited when all actual references are
                // dropped.
                let Some(state) = weak_state.upgrade() else {
                    return;
                };

                let now = Utc::now();
                state.tickets.lock().await.retain(|ticket, _| {
                    // SATEFY: ticket.timestamp was created using Utc::now().timestamp_millis(),
                    // which should generate a valid timestamp.
                    let age = now
                        - DateTime::<Utc>::from_timestamp_millis(ticket.timestamp)
                            .expect("current time is always a valid timestamp");
                    // Only keep the ticket if it was created less than 10 seconds ago.
                    age < Duration::seconds(10)
                });
            }
        });

        state
    }

    /// Creates a new ticket for a user and stores it.
    pub async fn new_ticket(&self, subject: Uuid, fingerprint: Fingerprint) -> Ticket {
        loop {
            let ticket = Ticket {
                subject,
                timestamp: Utc::now().timestamp_millis(),
                nonce: random(),
            };

            let mut mu = self.tickets.lock().await;
            // If the ticket already exists, retry (this will likely never happen though).
            if mu.contains_key(&ticket) {
                continue;
            }
            mu.insert(ticket.clone(), fingerprint);
            drop(mu);

            return ticket;
        }
    }

    /// Check if a ticket sent by a client is valid.
    pub async fn validate_response_ticket(
        &self,
        mut response: Ticket,
    ) -> Option<(Uuid, Fingerprint)> {
        // The client should have performed an operation on the nonce.
        response.nonce = response.nonce.wrapping_sub(1);

        let mut mu = self.tickets.lock().await;
        if let Some(fingerprint) = mu.remove(&response) {
            drop(mu);
            return Some((response.subject, fingerprint));
        }

        None
    }
}
