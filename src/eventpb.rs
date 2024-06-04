use core::fmt;

include!(concat!(env!("OUT_DIR"), "/eventpb.rs"));

impl EventMessage {
    /// Get the type of event.
    pub fn get_event_type(&self) -> Option<EventType> {
        let event = self.event.as_ref()?;

        match event {
            event_message::Event::MovementInfo(_) => Some(EventType::Movement),
            event_message::Event::InvadedInfo(_) => Some(EventType::Invaded),
        }
    }
}

/// The type of event.
pub enum EventType {
    /// Some creatures entered the room or is moving.
    Movement,
    /// A person is confirmed that he invaded into the room.
    Invaded,
}

impl fmt::Display for EventType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EventType::Movement => write!(f, "movement"),
            EventType::Invaded => write!(f, "invaded"),
        }
    }
}
