use bevy::prelude::*;

#[derive(Component)]
pub struct Ship;

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShipLocation {
    Docked {
        station: Entity,
    },
    InTransit {
        from: Entity,
        to: Entity,
        departed_at: u64,
        arrives_at: u64,
    },
}

impl ShipLocation {
    pub fn is_docked_at(&self, target: Entity) -> bool {
        matches!(self, Self::Docked { station } if *station == target)
    }

    pub fn arrival_unit(&self) -> Option<u64> {
        match self {
            Self::Docked { .. } => None,
            Self::InTransit { arrives_at, .. } => Some(*arrives_at),
        }
    }
}

#[derive(Message, Clone, Copy, Debug)]
pub struct TravelRequest {
    pub destination: Entity,
}

#[derive(Message, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShipArrived {
    pub ship: Entity,
    pub station: Entity,
    pub at_unit: u64,
}

#[derive(Resource, Default, Debug)]
pub struct ArrivalNotice(pub Option<String>);

#[derive(Resource, Debug)]
pub struct TravelResult(pub String);

impl Default for TravelResult {
    fn default() -> Self {
        Self("Choose a destination, then start time to travel.".to_string())
    }
}
