//! Iptscrae script event types.
//!
//! Events are triggered by user actions, room changes, or server events.
//! Scripts can register handlers for specific events using the ON keyword.

// Re-export EventMask from root - it's part of the wire protocol
pub use crate::EventMask;

/// Event type enumeration for matching individual events.
///
/// This enum provides a convenient way to work with individual event types
/// in the parser and AST, while EventMask is used for runtime event filtering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventType {
    Select,
    Lock,
    Unlock,
    Hide,
    Show,
    Startup,
    Alarm,
    Custom,
    InChat,
    PropChange,
    Enter,
    Leave,
    OutChat,
    SignOn,
    SignOff,
    Macro0,
    Macro1,
    Macro2,
    Macro3,
    Macro4,
    Macro5,
    Macro6,
    Macro7,
    Macro8,
    Macro9,
}

impl EventType {
    /// Convert event type to event mask
    pub const fn to_mask(self) -> EventMask {
        match self {
            EventType::Select => EventMask::SELECT,
            EventType::Lock => EventMask::LOCK,
            EventType::Unlock => EventMask::UNLOCK,
            EventType::Hide => EventMask::HIDE,
            EventType::Show => EventMask::SHOW,
            EventType::Startup => EventMask::STARTUP,
            EventType::Alarm => EventMask::ALARM,
            EventType::Custom => EventMask::CUSTOM,
            EventType::InChat => EventMask::INCHAT,
            EventType::PropChange => EventMask::PROPCHANGE,
            EventType::Enter => EventMask::ENTER,
            EventType::Leave => EventMask::LEAVE,
            EventType::OutChat => EventMask::OUTCHAT,
            EventType::SignOn => EventMask::SIGNON,
            EventType::SignOff => EventMask::SIGNOFF,
            EventType::Macro0 => EventMask::MACRO0,
            EventType::Macro1 => EventMask::MACRO1,
            EventType::Macro2 => EventMask::MACRO2,
            EventType::Macro3 => EventMask::MACRO3,
            EventType::Macro4 => EventMask::MACRO4,
            EventType::Macro5 => EventMask::MACRO5,
            EventType::Macro6 => EventMask::MACRO6,
            EventType::Macro7 => EventMask::MACRO7,
            EventType::Macro8 => EventMask::MACRO8,
            EventType::Macro9 => EventMask::MACRO9,
        }
    }

    /// Parse event name from string (case-insensitive)
    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_uppercase().as_str() {
            "SELECT" => Some(EventType::Select),
            "LOCK" => Some(EventType::Lock),
            "UNLOCK" => Some(EventType::Unlock),
            "HIDE" => Some(EventType::Hide),
            "SHOW" => Some(EventType::Show),
            "STARTUP" => Some(EventType::Startup),
            "ALARM" => Some(EventType::Alarm),
            "CUSTOM" => Some(EventType::Custom),
            "INCHAT" => Some(EventType::InChat),
            "PROPCHANGE" => Some(EventType::PropChange),
            "ENTER" => Some(EventType::Enter),
            "LEAVE" => Some(EventType::Leave),
            "OUTCHAT" => Some(EventType::OutChat),
            "SIGNON" => Some(EventType::SignOn),
            "SIGNOFF" => Some(EventType::SignOff),
            "MACRO0" => Some(EventType::Macro0),
            "MACRO1" => Some(EventType::Macro1),
            "MACRO2" => Some(EventType::Macro2),
            "MACRO3" => Some(EventType::Macro3),
            "MACRO4" => Some(EventType::Macro4),
            "MACRO5" => Some(EventType::Macro5),
            "MACRO6" => Some(EventType::Macro6),
            "MACRO7" => Some(EventType::Macro7),
            "MACRO8" => Some(EventType::Macro8),
            "MACRO9" => Some(EventType::Macro9),
            _ => None,
        }
    }

    /// Get event name as string
    pub const fn name(self) -> &'static str {
        match self {
            EventType::Select => "SELECT",
            EventType::Lock => "LOCK",
            EventType::Unlock => "UNLOCK",
            EventType::Hide => "HIDE",
            EventType::Show => "SHOW",
            EventType::Startup => "STARTUP",
            EventType::Alarm => "ALARM",
            EventType::Custom => "CUSTOM",
            EventType::InChat => "INCHAT",
            EventType::PropChange => "PROPCHANGE",
            EventType::Enter => "ENTER",
            EventType::Leave => "LEAVE",
            EventType::OutChat => "OUTCHAT",
            EventType::SignOn => "SIGNON",
            EventType::SignOff => "SIGNOFF",
            EventType::Macro0 => "MACRO0",
            EventType::Macro1 => "MACRO1",
            EventType::Macro2 => "MACRO2",
            EventType::Macro3 => "MACRO3",
            EventType::Macro4 => "MACRO4",
            EventType::Macro5 => "MACRO5",
            EventType::Macro6 => "MACRO6",
            EventType::Macro7 => "MACRO7",
            EventType::Macro8 => "MACRO8",
            EventType::Macro9 => "MACRO9",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_mask_values() {
        assert_eq!(EventMask::SELECT.bits(), 0x00000001);
        assert_eq!(EventMask::ENTER.bits(), 0x00000400);
        assert_eq!(EventMask::LEAVE.bits(), 0x00000800);
    }

    #[test]
    fn test_event_mask_operations() {
        let mask = EventMask::SELECT | EventMask::ENTER | EventMask::LEAVE;
        assert!(mask.contains(EventMask::SELECT));
        assert!(mask.contains(EventMask::ENTER));
        assert!(mask.contains(EventMask::LEAVE));
        assert!(!mask.contains(EventMask::LOCK));
    }

    #[test]
    fn test_event_mask_empty() {
        let mask = EventMask::empty();
        assert!(!mask.contains(EventMask::SELECT));
        assert_eq!(mask.bits(), 0);
    }

    #[test]
    fn test_event_mask_all() {
        let mask = EventMask::all();
        assert!(mask.contains(EventMask::SELECT));
        assert!(mask.contains(EventMask::ENTER));
        assert!(mask.contains(EventMask::MACRO9));
    }

    #[test]
    fn test_event_to_mask() {
        assert_eq!(EventType::Select.to_mask(), EventMask::SELECT);
        assert_eq!(EventType::Enter.to_mask(), EventMask::ENTER);
        assert_eq!(EventType::Leave.to_mask(), EventMask::LEAVE);
    }

    #[test]
    fn test_event_from_name() {
        assert_eq!(EventType::from_name("SELECT"), Some(EventType::Select));
        assert_eq!(EventType::from_name("select"), Some(EventType::Select));
        assert_eq!(EventType::from_name("ENTER"), Some(EventType::Enter));
        assert_eq!(EventType::from_name("invalid"), None);
    }

    #[test]
    fn test_event_name() {
        assert_eq!(EventType::Select.name(), "SELECT");
        assert_eq!(EventType::Enter.name(), "ENTER");
        assert_eq!(EventType::Macro5.name(), "MACRO5");
    }

    #[test]
    fn test_event_mask_default() {
        let mask = EventMask::default();
        assert!(mask.is_empty());
    }

    #[test]
    fn test_event_mask_from_bits() {
        let mask = EventMask::from_bits(0x0007).unwrap();
        assert!(mask.contains(EventMask::SELECT));
        assert!(mask.contains(EventMask::LOCK));
        assert!(mask.contains(EventMask::UNLOCK));
        assert!(!mask.contains(EventMask::HIDE));
    }

    #[test]
    fn test_event_mask_intersection() {
        let mask1 = EventMask::SELECT | EventMask::ENTER | EventMask::LEAVE;
        let mask2 = EventMask::ENTER | EventMask::LEAVE | EventMask::LOCK;
        let intersection = mask1 & mask2;
        assert!(!intersection.contains(EventMask::SELECT));
        assert!(intersection.contains(EventMask::ENTER));
        assert!(intersection.contains(EventMask::LEAVE));
        assert!(!intersection.contains(EventMask::LOCK));
    }

    #[test]
    fn test_event_mask_i32_conversion() {
        // Test Into<i32> and From<i32>
        let mask = EventMask::SELECT | EventMask::ENTER | EventMask::LEAVE;
        let i32_value: i32 = mask.into();
        assert_eq!(i32_value, 0x0C01); // 0x0001 | 0x0400 | 0x0800

        let mask2: EventMask = i32_value.into();
        assert_eq!(mask, mask2);
    }

    #[test]
    fn test_event_mask_from_trait() {
        // Test From<i32> trait
        let mask: EventMask = 0x0007i32.into(); // SELECT | LOCK | UNLOCK
        assert!(mask.contains(EventMask::SELECT));
        assert!(mask.contains(EventMask::LOCK));
        assert!(mask.contains(EventMask::UNLOCK));

        // Test Into<i32> trait
        let value: i32 = mask.into();
        assert_eq!(value, 0x0007);
    }

    #[test]
    fn test_event_mask_protocol_example() {
        // Test the example from room/records.rs: 0x0007 = SELECT | LOCK | UNLOCK
        let mask: EventMask = 0x0007i32.into();
        assert!(mask.contains(EventMask::SELECT));
        assert!(mask.contains(EventMask::LOCK));
        assert!(mask.contains(EventMask::UNLOCK));
        assert!(!mask.contains(EventMask::HIDE));
    }
}
