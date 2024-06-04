//! # IoT Edge System
//!
//! IoT Edge System is a edge component running on Raspberry Pi that
//! detects the environment data from the sensors, and send the
//! [`eventpb::EventMessage`] to [the central system](https://github.com/nkust-monitor-iot-project-2024/central).

pub mod eventpb;
pub mod mq;
